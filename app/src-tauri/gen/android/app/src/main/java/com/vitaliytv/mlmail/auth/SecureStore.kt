package com.vitaliytv.mlmail.auth

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey

object SecureStore {
    private const val PREFS = "mlmail_secure"
    private const val KEY_EMAIL = "google_email"
    private const val KEY_REFRESH = "google_refresh_token"

    private fun prefs(ctx: Context): SharedPreferences {
        val masterKey = MasterKey.Builder(ctx)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()
        return EncryptedSharedPreferences.create(
            ctx,
            PREFS,
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
        )
    }

    fun save(ctx: Context, email: String, refreshToken: String) {
        prefs(ctx).edit()
            .putString(KEY_EMAIL, email)
            .putString(KEY_REFRESH, refreshToken)
            .apply()
    }

    data class Loaded(val email: String?, val refreshToken: String?)

    fun load(ctx: Context): Loaded {
        val p = prefs(ctx)
        return Loaded(
            email = p.getString(KEY_EMAIL, null),
            refreshToken = p.getString(KEY_REFRESH, null),
        )
    }

    fun clear(ctx: Context) {
        prefs(ctx).edit().clear().apply()
    }
}
