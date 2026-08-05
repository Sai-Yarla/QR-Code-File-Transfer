package com.example.qrcodereceiver

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.example.qrcodereceiver.theme.QRCodeReceiverTheme
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class MainActivity : ComponentActivity() {
    private lateinit var decoderCore: DecoderCore
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Note: In a real environment, this needs the shared_core.so library compiled for Android.
        // decoderCore = DecoderCore()
        
        enableEdgeToEdge()
        setContent {
            QRCodeReceiverTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    MainScreen()
                }
            }
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        // decoderCore.destroy()
    }
}

@Composable
fun MainScreen() {
    var progress by remember { mutableStateOf(0.0f) }
    var isScanning by remember { mutableStateOf(false) }
    val coroutineScope = rememberCoroutineScope()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(
            text = "QR Code Receiver",
            style = MaterialTheme.typography.headlineMedium
        )
        Spacer(modifier = Modifier.height(32.dp))
        
        if (isScanning) {
            Text("Scanning... Please point camera at the screen.")
            Spacer(modifier = Modifier.height(16.dp))
            LinearProgressIndicator(
                progress = progress,
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(text = "${(progress * 100).toInt()}%")
            
            Spacer(modifier = Modifier.height(32.dp))
            Button(onClick = { isScanning = false }) {
                Text("Stop Scanning")
            }
        } else {
            Button(onClick = { 
                isScanning = true 
                // Start CameraX logic here
            }) {
                Text("Start Scanning")
            }
        }
    }
}
