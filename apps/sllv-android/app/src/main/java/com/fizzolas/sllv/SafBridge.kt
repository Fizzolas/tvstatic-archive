package com.fizzolas.sllv

import android.content.Context
import android.net.Uri
import android.provider.DocumentsContract
import androidx.documentfile.provider.DocumentFile
import java.io.File
import java.io.FileOutputStream

object SafBridge {

    fun cacheDir(context: Context): File {
        val d = File(context.cacheDir, "sllv")
        d.mkdirs()
        return d
    }

    fun framesWorkDir(context: Context): File {
        val d = File(context.filesDir, "frames")
        d.mkdirs()
        return d
    }

    fun clearDir(dir: File) {
        dir.listFiles()?.forEach {
            if (it.isDirectory) clearDir(it)
            it.delete()
        }
    }

    fun copyUriToFile(context: Context, uri: Uri, out: File) {
        context.contentResolver.openInputStream(uri).use { input ->
            requireNotNull(input)
            FileOutputStream(out).use { fos ->
                input.copyTo(fos)
            }
        }
    }

    fun copyTreeToDir(context: Context, treeUri: Uri, outDir: File) {
        val root = DocumentFile.fromTreeUri(context, treeUri) ?: return
        root.listFiles().forEach { child ->
            copyDocumentFile(context, child, outDir)
        }
    }

    private fun copyDocumentFile(context: Context, doc: DocumentFile, outDir: File) {
        if (doc.isDirectory) {
            val sub = File(outDir, doc.name ?: "dir")
            sub.mkdirs()
            doc.listFiles().forEach { c -> copyDocumentFile(context, c, sub) }
            return
        }
        val name = doc.name ?: "file.bin"
        val out = File(outDir, name)
        context.contentResolver.openInputStream(doc.uri).use { input ->
            requireNotNull(input)
            FileOutputStream(out).use { fos ->
                input.copyTo(fos)
            }
        }
    }
}
