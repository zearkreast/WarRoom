// Replaces the MainActivity that `tauri android init` generates; the Android workflow copies it in.
// The package must match `identifier` in tauri.conf.json.
package com.warcommand.tracker

import android.graphics.Color
import android.os.Bundle
import android.view.View
import androidx.activity.SystemBarStyle
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    // Light clock, battery and navigation icons, over the dark bars below.
    enableEdgeToEdge(
      statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
      navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
    )
    super.onCreate(savedInstanceState)

    // Edge to edge, the page would run under the status and navigation bars; padding keeps it between them.
    val content = findViewById<View>(android.R.id.content)
    content.setBackgroundColor(BAR_COLOR)
    ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
      val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout())
      view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      insets.inset(bars)
    }
  }

  private companion object {
    /** --surface-0 in src/ui/theme.css, the top bar's and dock's colour. */
    val BAR_COLOR = Color.parseColor("#0b0f16")
  }
}
