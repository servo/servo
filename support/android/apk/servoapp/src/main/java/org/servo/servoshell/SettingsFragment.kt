package org.servo.servoshell

import android.content.SharedPreferences
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ListItem
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.ComposeView
import androidx.compose.ui.res.stringResource
import androidx.core.content.edit
import androidx.fragment.app.Fragment
import androidx.preference.PreferenceManager

class SettingsFragment : Fragment() {
    private lateinit var preferences: SharedPreferences

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        preferences = PreferenceManager.getDefaultSharedPreferences(requireContext())
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?,
    ): View = inflater.inflate(R.layout.fragment_settings, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        view.findViewById<ComposeView>(R.id.toolbar).setContent {
            @OptIn(ExperimentalMaterial3Api::class)
            TopAppBar(
                title = { Text(stringResource(R.string.settings_title)) },
            )
        }
        view.findViewById<ComposeView>(R.id.body).setContent {
            Column {
                SettingsItem(
                    title = stringResource(R.string.settings_experimental_title),
                    summary = stringResource(R.string.settings_experimental_summary),
                    preferenceKey = "experimental",
                )
                SettingsItem(
                    title = stringResource(R.string.settings_animating_title),
                    summary = stringResource(R.string.settings_animating_summary),
                    preferenceKey = "animating_indicator",
                )
            }
        }
    }

    @Composable
    private fun SettingsItem(title: String, summary: String, preferenceKey: String) {
        ListItem(
            headlineContent = {
                Text(title)
            },
            supportingContent = {
                Text(summary)
            },
            trailingContent = {
                var checked by remember { mutableStateOf(preferences.getBoolean(preferenceKey, false)) }
                Switch(
                    checked = checked,
                    onCheckedChange = {
                        checked = it
                        preferences.edit { putBoolean(preferenceKey, it) }
                    },
                )
            },
        )
    }
}
