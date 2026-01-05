package com.fizzolas.sllv

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.view.View
import android.widget.Button
import android.widget.ProgressBar
import android.widget.TextView
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

class MainActivity : AppCompatActivity() {

    private var inputUri: Uri? = null
    private var inputIsFolder: Boolean = false

    private var framesDirUri: Uri? = null
    private var outTarUri: Uri? = null

    private lateinit var statusText: TextView
    private lateinit var progress: ProgressBar

    private val jni = NativeJni()

    private val pickInputFile = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri != null) {
            contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            inputUri = uri
            inputIsFolder = false
            status("Picked input file")
        }
    }

    private val pickInputFolder = registerForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri ->
        if (uri != null) {
            contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            inputUri = uri
            inputIsFolder = true
            status("Picked input folder")
        }
    }

    private val pickFramesFolder = registerForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri ->
        if (uri != null) {
            contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            framesDirUri = uri
            status("Picked frames folder")
        }
    }

    private val createOutputTar = registerForActivityResult(ActivityResultContracts.CreateDocument("application/x-tar")) { uri ->
        if (uri != null) {
            contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
            outTarUri = uri
            status("Output tar selected")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        statusText = findViewById(R.id.statusText)
        progress = findViewById(R.id.progressBar)

        findViewById<Button>(R.id.btnPickFile).setOnClickListener {
            pickInputFile.launch(arrayOf("*/*"))
        }
        findViewById<Button>(R.id.btnPickFolder).setOnClickListener {
            pickInputFolder.launch(null)
        }

        findViewById<Button>(R.id.btnEncode).setOnClickListener { startEncode() }

        findViewById<Button>(R.id.btnPickFramesIn).setOnClickListener {
            pickFramesFolder.launch(null)
        }
        findViewById<Button>(R.id.btnPickTarOut).setOnClickListener {
            createOutputTar.launch("recovered.tar")
        }
        findViewById<Button>(R.id.btnDecode).setOnClickListener { startDecode() }

        // Hide not-yet-implemented output frames picker (we encode to app-private frames dir for now).
        findViewById<View>(R.id.btnPickFramesOut).visibility = View.GONE
    }

    private fun startEncode() {
        val inUri = inputUri ?: run {
            status("Pick input file/folder first")
            return
        }

        setProgress(0)
        status("Preparing input...")

        CoroutineScope(Dispatchers.Main).launch {
            val ok = withContext(Dispatchers.IO) {
                try {
                    val cache = SafBridge.cacheDir(this@MainActivity)
                    SafBridge.clearDir(cache)

                    val stagedInput = if (!inputIsFolder) {
                        val f = File(cache, "input.bin")
                        SafBridge.copyUriToFile(this@MainActivity, inUri, f)
                        f
                    } else {
                        val d = File(cache, "input_folder")
                        d.mkdirs()
                        SafBridge.copyTreeToDir(this@MainActivity, inUri, d)
                        d
                    }

                    val frames = SafBridge.framesWorkDir(this@MainActivity)
                    frames.mkdirs()

                    // Call Rust via JNI on staged filesystem paths.
                    val rc = jni.packAndEncodeToFrames(stagedInput.absolutePath, frames.absolutePath)
                    rc == 0
                } catch (e: Exception) {
                    false
                }
            }

            if (ok) {
                setProgress(100)
                status("Encode done. Frames saved in app storage.")
            } else {
                setProgress(0)
                status("Encode failed")
            }
        }
    }

    private fun startDecode() {
        val framesUri = framesDirUri ?: run {
            status("Pick frames folder first")
            return
        }
        val outTar = outTarUri ?: run {
            status("Pick output tar first")
            return
        }

        setProgress(0)
        status("Staging frames...")

        CoroutineScope(Dispatchers.Main).launch {
            val ok = withContext(Dispatchers.IO) {
                try {
                    val cache = SafBridge.cacheDir(this@MainActivity)
                    val stagedFrames = File(cache, "frames_in")
                    stagedFrames.mkdirs()
                    SafBridge.clearDir(stagedFrames)
                    SafBridge.copyTreeToDir(this@MainActivity, framesUri, stagedFrames)

                    val outFile = File(cache, "recovered.tar")
                    if (outFile.exists()) outFile.delete()

                    val rc = jni.decodeFramesToTar(stagedFrames.absolutePath, outFile.absolutePath)
                    if (rc != 0) return@withContext false

                    // Copy tar bytes to the user-selected output URI.
                    contentResolver.openOutputStream(outTar).use { os ->
                        requireNotNull(os)
                        outFile.inputStream().use { it.copyTo(os) }
                    }

                    true
                } catch (e: Exception) {
                    false
                }
            }

            if (ok) {
                setProgress(100)
                status("Decode done. Output written.")
            } else {
                setProgress(0)
                status("Decode failed")
            }
        }
    }

    private fun setProgress(pct: Int) {
        progress.progress = pct
    }

    private fun status(msg: String) {
        statusText.text = msg
    }
}
