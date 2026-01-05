#include <jni.h>

// These must match the exported C ABI from the Rust cdylib.
extern int sllv_pack_and_encode_to_frames(const char* input_path, const char* out_dir);
extern int sllv_decode_frames_to_tar(const char* in_dir, const char* output_tar);

JNIEXPORT jint JNICALL
Java_com_fizzolas_sllv_SllvNative_packAndEncodeToFrames(JNIEnv* env, jclass clazz, jstring inputPath, jstring outDir) {
    const char* in = (*env)->GetStringUTFChars(env, inputPath, 0);
    const char* out = (*env)->GetStringUTFChars(env, outDir, 0);
    int rc = sllv_pack_and_encode_to_frames(in, out);
    (*env)->ReleaseStringUTFChars(env, inputPath, in);
    (*env)->ReleaseStringUTFChars(env, outDir, out);
    return (jint)rc;
}

JNIEXPORT jint JNICALL
Java_com_fizzolas_sllv_SllvNative_decodeFramesToTar(JNIEnv* env, jclass clazz, jstring inDir, jstring outputTar) {
    const char* in = (*env)->GetStringUTFChars(env, inDir, 0);
    const char* out = (*env)->GetStringUTFChars(env, outputTar, 0);
    int rc = sllv_decode_frames_to_tar(in, out);
    (*env)->ReleaseStringUTFChars(env, inDir, in);
    (*env)->ReleaseStringUTFChars(env, outputTar, out);
    return (jint)rc;
}
