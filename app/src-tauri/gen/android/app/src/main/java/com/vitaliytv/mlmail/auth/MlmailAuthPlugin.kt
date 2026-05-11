package com.vitaliytv.mlmail.auth

import android.app.Activity
import android.webkit.WebView
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

@InvokeArg
class SignInArgs {
    lateinit var webClientId: String
    lateinit var scopes: Array<String>
}

@InvokeArg
class SaveSessionArgs {
    lateinit var email: String
    lateinit var refreshToken: String
}

@TauriPlugin
class MlmailAuthPlugin(private val activity: Activity) : Plugin(activity) {

    override fun load(webView: WebView) {
        super.load(webView)
        (activity as? androidx.appcompat.app.AppCompatActivity)?.let { AuthorizationFlow.register(it) }
    }

    @Command
    fun signInAndAuthorize(invoke: Invoke) {
        val args = invoke.parseArgs(SignInArgs::class.java)
        CoroutineScope(Dispatchers.Main).launch {
            try {
                val cred = CredentialManagerFlow.signIn(activity, args.webClientId)
                val authz = AuthorizationFlow.authorize(activity, args.webClientId, args.scopes)
                val obj = JSObject()
                obj.put("server_auth_code", authz.serverAuthCode ?: "")
                obj.put("id_token", cred.idToken)
                obj.put("error", null)
                invoke.resolve(obj)
            } catch (e: Exception) {
                val obj = JSObject()
                obj.put("server_auth_code", null)
                obj.put("id_token", null)
                obj.put("error", classifyError(e))
                invoke.resolve(obj)
            }
        }
    }

    @Command
    fun saveSession(invoke: Invoke) {
        val args = invoke.parseArgs(SaveSessionArgs::class.java)
        SecureStore.save(activity, args.email, args.refreshToken)
        invoke.resolve()
    }

    @Command
    fun loadSession(invoke: Invoke) {
        val loaded = SecureStore.load(activity)
        val obj = JSObject()
        obj.put("email", loaded.email)
        obj.put("refresh_token", loaded.refreshToken)
        invoke.resolve(obj)
    }

    @Command
    fun clearSession(invoke: Invoke) {
        SecureStore.clear(activity)
        invoke.resolve()
    }

    private fun classifyError(e: Exception): String {
        val name = e::class.java.simpleName
        return if (name.contains("Cancel", ignoreCase = true)) "Cancelled" else (e.message ?: name)
    }
}
