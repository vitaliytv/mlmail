package com.vitaliytv.mlmail.auth

import android.app.Activity
import android.content.Intent
import androidx.activity.result.ActivityResult
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.IntentSenderRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import com.google.android.gms.auth.api.identity.AuthorizationRequest
import com.google.android.gms.auth.api.identity.AuthorizationResult
import com.google.android.gms.auth.api.identity.Identity
import com.google.android.gms.common.api.Scope
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.tasks.await

object AuthorizationFlow {
    private var pendingDeferred: CompletableDeferred<AuthorizationResult>? = null
    private var launcher: ActivityResultLauncher<IntentSenderRequest>? = null

    fun register(activity: AppCompatActivity) {
        launcher = activity.registerForActivityResult(
            ActivityResultContracts.StartIntentSenderForResult()
        ) { result: ActivityResult ->
            val def = pendingDeferred
            pendingDeferred = null
            if (def == null) return@registerForActivityResult
            try {
                val authResult = Identity.getAuthorizationClient(activity)
                    .getAuthorizationResultFromIntent(result.data)
                def.complete(authResult)
            } catch (e: Exception) {
                def.completeExceptionally(e)
            }
        }
    }

    suspend fun authorize(
        activity: Activity,
        webClientId: String,
        scopes: Array<String>,
    ): AuthorizationResult {
        val request = AuthorizationRequest.Builder()
            .setRequestedScopes(scopes.map { Scope(it) })
            .requestOfflineAccess(webClientId, /* forceCodeForRefreshToken = */ true)
            .build()

        val client = Identity.getAuthorizationClient(activity)
        val result = client.authorize(request).await()

        if (result.hasResolution()) {
            val pendingIntent = result.pendingIntent
                ?: throw IllegalStateException("authorization needs resolution but pendingIntent is null")
            val l = launcher
                ?: throw IllegalStateException("AuthorizationFlow.register() must be called from MainActivity.onCreate")
            val deferred = CompletableDeferred<AuthorizationResult>()
            pendingDeferred = deferred
            val intentSender = IntentSenderRequest.Builder(pendingIntent.intentSender).build()
            l.launch(intentSender)
            return deferred.await()
        }
        return result
    }
}
