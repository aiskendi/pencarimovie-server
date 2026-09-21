package com.pencarimovie.downloader;

import android.app.Application;
import android.util.Log;

public class ServerApplication extends Application {

    private static final String LOG_TAG = "ServerApplication";

    @Override
    public void onCreate() {
        super.onCreate();
        Log.i(LOG_TAG, "PencariMovie Downloader application starting");
    }
}
