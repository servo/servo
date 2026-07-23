/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
package org.servo.servoshell

import android.content.SharedPreferences
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.core.content.edit
import androidx.preference.PreferenceManager

class SettingsActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val preferences = PreferenceManager.getDefaultSharedPreferences(this)
        setContent {
            Scaffold(
                topBar = {
                    @OptIn(ExperimentalMaterial3Api::class)
                    TopAppBar(
                        title = { Text(stringResource(R.string.settings_title)) },
                        navigationIcon = {
                            IconButton(onClick = { finish() }) {
                                Icon(painterResource(R.drawable.arrow_back), stringResource(R.string.back))
                            }
                        },
                    )
                },
            ) { innerPadding ->
                Column(modifier = Modifier.padding(innerPadding)) {
                    SettingsItem(
                        title = stringResource(R.string.settings_experimental_title),
                        summary = stringResource(R.string.settings_experimental_summary),
                        preferences = preferences,
                        preferenceKey = "experimental",
                    )
                }
            }
        }
    }

    @Composable
    private fun SettingsItem(title: String, summary: String, preferences: SharedPreferences, preferenceKey: String) {
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
