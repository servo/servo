package org.servo.servoshell;

import android.content.SharedPreferences;
import android.os.Bundle;
import android.preference.PreferenceManager;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.fragment.app.Fragment;
import com.google.android.material.switchmaterial.SwitchMaterial;

public class SettingsFragment extends Fragment {
    
    private SharedPreferences mPreferences;
    private SwitchMaterial mExperimentalSwitch;
    private SwitchMaterial mAnimatingSwitch;
    
    @Override
    public void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        mPreferences = PreferenceManager.getDefaultSharedPreferences(requireContext());
    }
    
    @Nullable
    @Override
    public View onCreateView(@NonNull LayoutInflater inflater, @Nullable ViewGroup container,
                             @Nullable Bundle savedInstanceState) {
        // Inflate the new Material3 layout
        return inflater.inflate(R.layout.fragment_settings, container, false);
    }
    
    @Override
    public void onViewCreated(@NonNull View view, @Nullable Bundle savedInstanceState) {
        super.onViewCreated(view, savedInstanceState);
        
        // Find the switches and containers
        mExperimentalSwitch = view.findViewById(R.id.experimental_switch);
        mAnimatingSwitch = view.findViewById(R.id.animating_switch);
        View experimentalContainer = view.findViewById(R.id.experimental_container);
        View animatingContainer = view.findViewById(R.id.animating_container);
        
        // Load current preference values
        loadPreferences();
        
        // Set up click listeners for the entire row
        experimentalContainer.setOnClickListener(v -> {
            boolean newValue = !mExperimentalSwitch.isChecked();
            mExperimentalSwitch.setChecked(newValue);
            savePreference("experimental", newValue);
        });
        
        animatingContainer.setOnClickListener(v -> {
            boolean newValue = !mAnimatingSwitch.isChecked();
            mAnimatingSwitch.setChecked(newValue);
            savePreference("animating_indicator", newValue);
        });
        
        // Switches need extra listeners, apparently
        mExperimentalSwitch.setOnCheckedChangeListener((buttonView, isChecked) -> {
            // Only save if the change was initiated by user interaction
            if (buttonView.isPressed()) {
                savePreference("experimental", isChecked);
            }
        });
        
        mAnimatingSwitch.setOnCheckedChangeListener((buttonView, isChecked) -> {
            if (buttonView.isPressed()) {
                savePreference("animating_indicator", isChecked);
            }
        });
    }
    

    private void loadPreferences() {
        boolean experimental = mPreferences.getBoolean("experimental", false);
        boolean animatingIndicator = mPreferences.getBoolean("animating_indicator", false);
        
        // Update switch states
        mExperimentalSwitch.setChecked(experimental);
        mAnimatingSwitch.setChecked(animatingIndicator);
    }
    
    private void savePreference(String key, boolean value) {
        mPreferences.edit().putBoolean(key, value).apply();
    }
}
