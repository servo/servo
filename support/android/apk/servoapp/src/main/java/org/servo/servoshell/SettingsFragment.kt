package org.servo.servoshell

import android.content.SharedPreferences
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.fragment.app.Fragment
import androidx.preference.PreferenceManager
import com.google.android.material.switchmaterial.SwitchMaterial
import androidx.core.content.edit

class SettingsFragment : Fragment() {
    private lateinit var preferences: SharedPreferences
    private lateinit var experimentalSwitch: SwitchMaterial
    private lateinit var animatingSwitch: SwitchMaterial

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
        experimentalSwitch = view.findViewById(R.id.experimental_switch)
        animatingSwitch = view.findViewById(R.id.animating_switch)
        val experimentalContainer = view.findViewById<View>(R.id.experimental_container)
        val animatingContainer = view.findViewById<View>(R.id.animating_container)

        loadPreferences()

        experimentalContainer.setOnClickListener {
            val newValue = !experimentalSwitch.isChecked
            experimentalSwitch.isChecked = newValue
            savePreference("experimental", newValue)
        }

        animatingContainer.setOnClickListener {
            val newValue = !animatingSwitch.isChecked
            animatingSwitch.isChecked = newValue
            savePreference("animating_indicator", newValue)
        }

        experimentalSwitch.setOnCheckedChangeListener { buttonView, isChecked ->
            if (buttonView.isPressed) {
                savePreference("experimental", isChecked)
            }
        }

        animatingSwitch.setOnCheckedChangeListener { buttonView, isChecked ->
            if (buttonView.isPressed) {
                savePreference("animating_indicator", isChecked)
            }
        }
    }

    private fun loadPreferences() {
        val experimental = preferences.getBoolean("experimental", false)
        val animatingIndicator = preferences.getBoolean("animating_indicator", false)

        experimentalSwitch.isChecked = experimental
        animatingSwitch.isChecked = animatingIndicator
    }

    private fun savePreference(key: String, value: Boolean) {
        preferences.edit { putBoolean(key, value) }
    }
}
