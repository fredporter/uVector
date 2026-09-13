import os
import sys
import unittest

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from python.uvector_banana import (
    STYLE_PRESETS,
    NanoBananaClient,
    build_banana_prompt,
    load_canonical_palettes,
    get_base_presets,
)


class TestNanoBanana(unittest.TestCase):
    def test_presets_exist(self):
        self.assertIn("mono_teletext", STYLE_PRESETS)
        self.assertIn("mono_blueprint", STYLE_PRESETS)
        self.assertIn("mono_paper", STYLE_PRESETS)
        self.assertIn("pixel_art", STYLE_PRESETS)
        self.assertIn("line_art", STYLE_PRESETS)
        # Ensure Amber CRT is NOT present
        self.assertNotIn("mono_amber", STYLE_PRESETS)

    def test_canonical_palettes_loaded(self):
        reg = load_canonical_palettes()
        self.assertIn("palettes", reg)
        palettes = reg["palettes"]
        self.assertIn("teletext_ceefax", palettes)
        self.assertIn("architectural_blueprint", palettes)
        self.assertNotIn("mono_amber", palettes)
        self.assertNotIn("amber_crt", palettes)

    def test_build_banana_prompt(self):
        prompt = build_banana_prompt("uDOS architecture terminal", style_preset="mono_teletext")
        self.assertIn("uDOS architecture terminal", prompt)
        self.assertIn("teletext", prompt)
        self.assertIn("#000000", prompt)

        bp_prompt = build_banana_prompt("Space station", style_preset="mono_blueprint")
        self.assertIn("Space station", bp_prompt)
        self.assertIn("blueprint", bp_prompt)
        self.assertIn("#0a2342", bp_prompt)

    def test_client_fallback_offline(self):
        # Offline endpoint that will fail and use graceful fallback
        client = NanoBananaClient(base_url="http://127.0.0.1:59999", timeout=0.1)
        res = client.generate("Cyberpunk skyline", style_preset="mono_blueprint")
        self.assertEqual(res["status"], "success")
        self.assertEqual(res["model"], "imagen-3.0-generate-002")
        self.assertEqual(res["style_preset"], "mono_blueprint")
        self.assertEqual(res["aspect_ratio"], "16:9")
        self.assertTrue(res.get("offline_fallback"))
        self.assertIn("nano_banana", res["asset_id"])


if __name__ == "__main__":
    unittest.main()
