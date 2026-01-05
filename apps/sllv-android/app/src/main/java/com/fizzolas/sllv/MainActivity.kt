package com.fizzolas.sllv

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.DocumentsContract
import android.view.View
import android.widget.Button
import android.widget.ProgressBar
import android.widget.TextView
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {

    private var inputUri: Uri? = null
    private var outDirUri: Uri? = null
    private var framesDirUri: Uri? = null
    private var outTarUri: Uri? = null

    private lateinit var statusText: TextView
    private lateinit var progress: ProgressBar

    private val pickInputFile = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri != null) {
            contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            inputUri = uri
            status("Picked input file: $uri")
        }
    }

    private val pickInputFolder = registerForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri ->
        if (uri != null) {
            contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            inputUri = uri
            status("Picked input folder: $uri")
        }
    }

    private val pickOutputFramesFolder = registerForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri ->
        if (uri != null) {
            contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
            outDirUri = uri
            status("Picked output frames folder: $uri")
        }
    }

    private val pickFramesFolder = registerForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri ->
        if (uri != null) {
            contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            framesDirUri = uri
            status("Picked frames folder: $uri")
        }
    }

    private val createOutputTar = registerForActivityResult(ActivityResultContracts.CreateDocument("application/x-tar")) { uri ->
        if (uri != null) {
            contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
            outTarUri = uri
            status("Output tar: $uri")
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
            // ACTION_OPEN_DOCUMENT_TREE is the recommended SAF route for directory selection. [web:118]
            pickInputFolder.launch(null)
        }

        findViewById<Button>(R.id.btnPickFramesOut).setOnClickListener {
            pickOutputFramesFolder.launch(null)
        }

        findViewById<Button>(R.id.btnEncode).setOnClickListener {
            startEncode()
        }

        findViewById<Button>(R.id.btnPickFramesIn).setOnClickListener {
            pickFramesFolder.launch(null)
        }

        findViewById<Button>(R.id.btnPickTarOut).setOnClickListener {
            createOutputTar.launch("recovered.tar")
        }

        findViewById<Button>(R.id.btnDecode).setOnClickListener {
            startDecode()
        }
    }

    private fun startEncode() {
        val inUri = inputUri
        val outUri = outDirUri
        if (inUri == null || outUri == null) {
            status("Pick input + output frames folder first")
            return
        }

        // SAF URIs aren't filesystem paths; next increment will copy to cache and encode from there.
        status("Encode: SAF URI copy not implemented yet (next increment)")
    }

    private fun startDecode() {
        val framesUri = framesDirUri
        val outUri = outTarUri
        if (framesUri == null || outUri == null) {
            status("Pick frames folder + output tar first")
            return
        }

        status("Decode: SAF URI copy not implemented yet (next increment)")
    }

    private fun status(msg: String) {
        statusText.text = msg
    }
}
