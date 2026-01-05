package com.fizzolas.sllv;

public class SllvNative {
  static {
    System.loadLibrary("sllv_ffi");
  }

  public static native int packAndEncodeToFrames(String inputPath, String outDir);
  public static native int decodeFramesToTar(String inDir, String outputTar);
}
