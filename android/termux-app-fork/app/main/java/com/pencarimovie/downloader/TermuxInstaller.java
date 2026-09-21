package com.pencarimovie.downloader;

import android.app.Activity;
import android.app.AlertDialog;
import android.app.ProgressDialog;
import android.content.Context;
import android.system.Os;
import android.util.Log;
import android.view.WindowManager;

import java.io.BufferedReader;
import java.io.ByteArrayInputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.util.ArrayList;
import java.util.List;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

/**
 * Install the PencariMovie bootstrap packages if necessary.
 *
 * The bootstrap is a custom Termux build with
 * `TERMUX_APP__PACKAGE_NAME="com.pencarimovie.downloader"`, so all
 * binaries have hardcoded paths to our data directory and run natively
 * on Android (bionic libc + /system/bin/linker64) — no proot needed.
 *
 * The bootstrap includes: bash, coreutils, PHP (with curl, mbstring, openssl,
 * sodium, zip, bz2, gmp, intl, pgsql, gd, fpm, pcntl, bcmath, sockets),
 * wget, proot, and other essential utilities.
 *
 * Steps:
 * 1. If $PREFIX already exists, assume it's correct and finish.
 * 2. Show progress dialog.
 * 3. Delete staging prefix directory.
 * 4. Delete prefix directory.
 * 5. Create staging prefix directory.
 * 6. Load bootstrap ZIP from libpencarimovie-bootstrap.so (NDK embedded)
 *    or fall back to assets/bootstrap-{arch}.zip.
 * 7. Extract ZIP into staging prefix directory.
 * 8. Process SYMLINKS.txt.
 * 9. Rename staging -> prefix.
 * 10. Done.
 */
final class TermuxInstaller {

    private static final String LOG_TAG = "PencariMovieInstaller";

    /** The $PREFIX directory path for our app. */
    static final String PREFIX_DIR_PATH =
        "/data/data/com.pencarimovie.downloader/files/usr";

    /** The staging prefix directory path. */
    static final String STAGING_PREFIX_DIR_PATH =
        PREFIX_DIR_PATH + ".staging";

    /** Performs bootstrap setup if necessary. */
    static void setupBootstrapIfNeeded(final Activity activity, final Runnable whenDone) {
        // If prefix directory exists, assume it's correct
        File prefixDir = new File(PREFIX_DIR_PATH);
        if (prefixDir.isDirectory()) {
            // Check if it's not empty
            String[] contents = prefixDir.list();
            if (contents != null && contents.length > 0) {
                Log.i(LOG_TAG, "Prefix directory already exists and is not empty, skipping bootstrap");
                whenDone.run();
                return;
            }
        }

        final ProgressDialog progress = ProgressDialog.show(activity, null,
            "Installing bootstrap...", true, false);

        new Thread() {
            @Override
            public void run() {
                try {
                    Log.i(LOG_TAG, "Installing PencariMovie bootstrap packages.");

                    File stagingDir = new File(STAGING_PREFIX_DIR_PATH);
                    File targetPrefixDir = new File(PREFIX_DIR_PATH);

                    // Ensure parent directory (/data/data/com.pencarimovie.downloader/files) exists
                    File filesDir = stagingDir.getParentFile();
                    if (filesDir != null && !filesDir.exists()) {
                        filesDir.mkdirs();
                    }

                    // Delete staging prefix directory if it exists
                    if (stagingDir.exists()) {
                        deleteRecursive(stagingDir);
                    }

                    // Delete prefix directory if it exists
                    if (targetPrefixDir.exists()) {
                        deleteRecursive(targetPrefixDir);
                    }

                    // Create staging prefix directory
                    if (!stagingDir.exists() && !stagingDir.mkdirs()) {
                        throw new RuntimeException("Failed to create staging directory: " + STAGING_PREFIX_DIR_PATH);
                    }

                    Log.i(LOG_TAG, "Extracting bootstrap zip to staging directory.");

                    final byte[] buffer = new byte[8096];
                    final List<String[]> symlinks = new ArrayList<>(50);

                    // Stream zip directly from source — do NOT load entire 137 MB into memory
                    extractBootstrapZip(stagingDir, buffer, symlinks);

                    // Note: The Termux bootstrap has many symlinks (bin/sh → bash, etc.)
                    // which are recorded in SYMLINKS.txt and recreated after extraction.
                    for (String[] symlink : symlinks) {
                        Os.symlink(symlink[0], symlink[1]);
                    }

                    // No .deb extraction needed — wget, proot are all in the bootstrap (PHP is in the FrankenPHP release tarball).
                    // No path patching needed — bootstrap was compiled with
                    // TERMUX_APP__PACKAGE_NAME="com.pencarimovie.downloader".

                    Log.i(LOG_TAG, "Moving staging to prefix directory.");

                    if (!stagingDir.renameTo(targetPrefixDir)) {
                        throw new RuntimeException("Moving staging to prefix directory failed");
                    }

                    Log.i(LOG_TAG, "Bootstrap packages installed successfully.");

                    activity.runOnUiThread(whenDone);

                } catch (final Exception e) {
                    showBootstrapErrorDialog(activity, whenDone,
                        Log.getStackTraceString(e));
                } finally {
                    activity.runOnUiThread(() -> {
                        try {
                            progress.dismiss();
                        } catch (RuntimeException ignored) {
                            // Activity already dismissed
                        }
                    });
                }
            }
        }.start();
    }

    private static void showBootstrapErrorDialog(Activity activity, Runnable whenDone, String message) {
        Log.e(LOG_TAG, "Bootstrap Error:\n" + message);

        activity.runOnUiThread(() -> {
            try {
                new AlertDialog.Builder(activity)
                    .setTitle("Bootstrap Error")
                    .setMessage("Failed to install bootstrap packages:\n\n" + message)
                    .setNegativeButton("Abort", (dialog, which) -> {
                        dialog.dismiss();
                        activity.finish();
                    })
                    .setPositiveButton("Try Again", (dialog, which) -> {
                        dialog.dismiss();
                        TermuxInstaller.setupBootstrapIfNeeded(activity, whenDone);
                    }).show();
            } catch (WindowManager.BadTokenException ignored) {
                // Activity already dismissed
            }
        });
    }

    private static void deleteRecursive(File file) {
        if (file.isDirectory()) {
            File[] children = file.listFiles();
            if (children != null) {
                for (File child : children) {
                    deleteRecursive(child);
                }
            }
        }
        file.delete();
    }

    /**
     * Extract bootstrap ZIP to staging directory, streaming directly from the source
     * to avoid loading the entire (244 MB) ZIP into memory.
     *
     * Preferred: native library (libpencarimovie-bootstrap.so) via loadZipBytes().
     * Fallback: stream directly from assets using AssetManager.open() + ZipInputStream.
     */
    private static void extractBootstrapZip(
            File stagingDir, byte[] buffer, List<String[]> symlinks) throws Exception {

        // Try native library first (fast, no heap spike)
        try {
            System.loadLibrary("pencarimovie-bootstrap");
            byte[] zipBytes = getZip();
            Log.i(LOG_TAG, "Loaded bootstrap from native library (" + zipBytes.length + " bytes)");
            try (ZipInputStream zipInput = new ZipInputStream(new ByteArrayInputStream(zipBytes))) {
                extractZipEntries(zipInput, stagingDir, buffer, symlinks);
            }
            return;
        } catch (UnsatisfiedLinkError e) {
            Log.w(LOG_TAG, "Native library not available, streaming from assets: " + e.getMessage());
        }

        // Fallback: stream directly from assets — no byte[] allocation of the full ZIP
        Context ctx = null;
        try {
            Class<?> activityThread = Class.forName("android.app.ActivityThread");
            java.lang.reflect.Method method = activityThread.getMethod("currentApplication");
            ctx = (android.content.Context) method.invoke(null);
        } catch (Exception ignored) {
        }
        if (ctx == null) {
            throw new RuntimeException("Cannot extract bootstrap: no native library and no context available");
        }

        String arch = System.getProperty("os.arch", "aarch64");
        if (arch.contains("64")) {
            if (arch.contains("x86") || arch.contains("amd"))
                arch = "x86_64";
            else
                arch = "aarch64";
        } else if (arch.contains("86")) {
            arch = "i686";
        } else if (arch.contains("arm") || arch.contains("v7")) {
            arch = "arm";
        }
        String assetName = "bootstrap-" + arch + ".zip";
        Log.i(LOG_TAG, "Streaming bootstrap from assets: " + assetName);

        try (InputStream is = ctx.getAssets().open(assetName);
             ZipInputStream zipInput = new ZipInputStream(is)) {
            extractZipEntries(zipInput, stagingDir, buffer, symlinks);
        }
    }

    /** Common ZIP extraction logic used by both native and asset paths. */
    private static void extractZipEntries(
            ZipInputStream zipInput, File stagingDir, byte[] buffer, List<String[]> symlinks) throws Exception {
        ZipEntry zipEntry;
        while ((zipEntry = zipInput.getNextEntry()) != null) {
            if (zipEntry.getName().equals("SYMLINKS.txt")) {
                BufferedReader symlinksReader = new BufferedReader(new InputStreamReader(zipInput));
                String line;
                while ((line = symlinksReader.readLine()) != null) {
                    String[] parts = line.split("←");
                    if (parts.length != 2)
                        throw new RuntimeException("Malformed symlink line: " + line);
                    String oldPath = parts[0];
                    String newPath = stagingDir.getAbsolutePath() + "/" + parts[1];
                    symlinks.add(new String[]{oldPath, newPath});

                    File parent = new File(newPath).getParentFile();
                    if (parent != null && !parent.exists()) {
                        parent.mkdirs();
                    }
                }
            } else {
                String zipEntryName = zipEntry.getName();
                File targetFile = new File(stagingDir, zipEntryName);
                boolean isDirectory = zipEntry.isDirectory();

                File parent = isDirectory ? targetFile : targetFile.getParentFile();
                if (parent != null && !parent.exists()) {
                    parent.mkdirs();
                }

                // If the entry is bin/su, skip it
                if (zipEntryName.equals("bin/su") || zipEntryName.equals("bin/su/")) {
                    Log.i(LOG_TAG, "Skipping su binary from bootstrap");
                    continue;
                }

                if (isDirectory) {
                    targetFile.mkdirs();
                } else {
                    if (targetFile.exists()) {
                        if (targetFile.isDirectory()) {
                            deleteRecursive(targetFile);
                        } else {
                            targetFile.delete();
                        }
                    }
                    if (parent != null && !parent.exists()) {
                        parent.mkdirs();
                    }
                    try {
                        try (FileOutputStream outStream = new FileOutputStream(targetFile)) {
                            int readBytes;
                            while ((readBytes = zipInput.read(buffer)) != -1)
                                outStream.write(buffer, 0, readBytes);
                        }
                        // Set executable permissions on bin/, libexec, and lib/apt/methods entries
                        if (zipEntryName.startsWith("bin/") ||
                            zipEntryName.startsWith("libexec") ||
                            zipEntryName.startsWith("lib/apt/methods/")) {
                            //noinspection OctalInteger
                            Os.chmod(targetFile.getAbsolutePath(), 0755);
                        }
                    } catch (Exception e) {
                        Log.w(LOG_TAG, "Could not extract entry " + zipEntryName + ": " + e.getMessage());
                        // If it's not a critical core binary, don't abort entire bootstrap
                        if (!zipEntryName.equals("bin/bash") && !zipEntryName.equals("bin/sh")) {
                            Log.w(LOG_TAG, "Ignoring non-critical extraction failure for " + zipEntryName);
                        } else {
                            throw e;
                        }
                    }
                }
            }
        }
    }

    /**
     * Load the bootstrap ZIP bytes.
     *
     * Preferred method: load from the embedded native library (libpencarimovie-bootstrap.so).
     * Fallback: read from assets/bootstrap-aarch64.zip (useful when NDK is not installed).
     *
     * WARNING: Only use this when the returned byte[] fits in heap. For the 244 MB bootstrap,
     * prefer extractBootstrapZip() which streams directly instead.
     */
    public static byte[] loadZipBytes() {
        // Try native library first (embedded via NDK .incbin)
        try {
            System.loadLibrary("pencarimovie-bootstrap");
            return getZip();
        } catch (UnsatisfiedLinkError e) {
            Log.w(LOG_TAG, "Native library not available, falling back to assets: " + e.getMessage());
        }

        // Fallback: read from assets
        try {
            Context ctx = null;
            // Try to get a context via activity thread
            try {
                Class<?> activityThread = Class.forName("android.app.ActivityThread");
                java.lang.reflect.Method method = activityThread.getMethod("currentApplication");
                ctx = (android.content.Context) method.invoke(null);
            } catch (Exception ignored) {
            }
            if (ctx == null) {
                throw new RuntimeException("Cannot load bootstrap: no native library and no context available");
            }
            String arch = System.getProperty("os.arch", "aarch64");
            // Normalize arch names
            if (arch.contains("64")) {
                if (arch.contains("x86") || arch.contains("amd"))
                    arch = "x86_64";
                else
                    arch = "aarch64";
            } else if (arch.contains("86")) {
                arch = "i686";
            } else if (arch.contains("arm") || arch.contains("v7")) {
                arch = "arm";
            }
            String assetName = "bootstrap-" + arch + ".zip";
            Log.i(LOG_TAG, "Loading bootstrap from assets: " + assetName);
            try (java.io.InputStream is = ctx.getAssets().open(assetName)) {
                // Read the full stream — do NOT rely on available() which may under-report
                java.io.ByteArrayOutputStream baos = new java.io.ByteArrayOutputStream();
                byte[] buffer = new byte[8192];
                int read;
                while ((read = is.read(buffer)) != -1) {
                    baos.write(buffer, 0, read);
                }
                byte[] data = baos.toByteArray();
                Log.i(LOG_TAG, "Loaded " + data.length + " bytes from assets/" + assetName);
                return data;
            }
        } catch (java.io.IOException e) {
            throw new RuntimeException("Failed to load bootstrap from assets: " + e.getMessage(), e);
        }
    }

    public static native byte[] getZip();
}
