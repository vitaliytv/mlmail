package com.vitaliytv.mlmail.auth

import android.app.Activity
import androidx.credentials.CredentialManager
import androidx.credentials.GetCredentialRequest
import com.google.android.libraries.identity.googleid.GetGoogleIdOption
import com.google.android.libraries.identity.googleid.GoogleIdTokenCredential

object CredentialManagerFlow {
    suspend fun signIn(activity: Activity, webClientId: String): GoogleIdTokenCredential {
        val option = GetGoogleIdOption.Builder()
            .setServerClientId(webClientId)
            .setFilterByAuthorizedAccounts(false)
            .build()

        val request = GetCredentialRequest.Builder()
            .addCredentialOption(option)
            .build()

        val result = CredentialManager.create(activity).getCredential(activity, request)
        return GoogleIdTokenCredential.createFrom(result.credential.data)
    }
}
