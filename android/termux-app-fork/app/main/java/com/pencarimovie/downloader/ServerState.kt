package com.pencarimovie.downloader

sealed class ServerState {
    data object Idle : ServerState()
    data object Starting : ServerState()
    /** Setup phase: pkg install, downloading release, extracting. */
    data class SetupProgress(val message: String) : ServerState()
    data class Running(val host: String, val port: Int, val lanIp: String?) : ServerState()
    data class Error(val message: String) : ServerState()
    data object Stopping : ServerState()
}
