cmake_minimum_required(VERSION 3.22.1)
project(sllv)

add_library(sllv_jni SHARED sllv_jni.c)

# The Rust library is provided via jniLibs as libsllv_ffi.so.
# We link against it by declaring it as an imported library.
add_library(sllv_ffi SHARED IMPORTED)

set_target_properties(sllv_ffi PROPERTIES
    IMPORTED_NO_SONAME TRUE
    IMPORTED_LOCATION "${CMAKE_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}/libsllv_ffi.so")

target_link_libraries(sllv_jni sllv_ffi)
