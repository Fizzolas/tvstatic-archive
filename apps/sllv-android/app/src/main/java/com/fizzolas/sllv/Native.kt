package com.fizzolas.sllv

object Native {
    init {
        System.loadLibrary("sllv_ffi")
    }

    external fun packAndEncodeToFrames(inputPath: String, outDir: String): Int
    external fun decodeFramesToTar(inDir: String, outputTar: String): Int
}
