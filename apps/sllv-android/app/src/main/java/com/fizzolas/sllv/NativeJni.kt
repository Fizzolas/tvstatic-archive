package com.fizzolas.sllv

class NativeJni {
    companion object {
        init {
            System.loadLibrary("sllv_ffi")
        }
    }

    external fun packAndEncodeToFrames(inputPath: String, outDir: String): Int
    external fun decodeFramesToTar(inDir: String, outputTar: String): Int
}
