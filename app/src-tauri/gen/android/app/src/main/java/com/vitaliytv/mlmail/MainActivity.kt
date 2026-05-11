package com.vitaliytv.mlmail

import android.os.Bundle
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    // AuthorizationFlow.register requires AppCompatActivity. TauriActivity
    // is built on AppCompatActivity, so this cast is safe at runtime.
    if (this is androidx.appcompat.app.AppCompatActivity) {
      com.vitaliytv.mlmail.auth.AuthorizationFlow.register(this)
    }
  }
}
