import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_desktop import CLAIMS, EVENTS, assess, validate
from check_p0_gui import check_native_ui_features


def fixture():
    return {"schema_version": 1, "status": "recorded", "source": "native_user_session",
            "clock": "synthetic desktop fixture, not device audio or gameplay", "simulation_us": 10000,
            "paused": True, "history_nodes": 2, "position": [120, 140], "draft_active": False,
            "accepted_game_inputs": 1, "native_event_observations": dict.fromkeys(EVENTS, 1),
            "text_summary": {"characters": 9, "matches_fixture": True}, "layouts_visited": [True, True],
            "scale_observations": [{"native_pixels_per_point": 1.25, "effective_pixels_per_point": 1.25, "ui_zoom": 1.0}],
            "pending_fullscreen": None, "human_confirmations": dict.fromkeys(CLAIMS, True),
            "unavailable": {"japanese_ime": False, "cross_native_dpi": False}, "errors": [], "limits": [],
            "elapsed_ms": 10000, "ended_by": "record_button", "system_capture_used": False,
            "input_text_persisted": False, "foreign_file_bytes_read": False}


class DesktopTests(unittest.TestCase):
    def test_native_clipboard_cannot_silently_fall_back_to_application_only(self):
        features = ["default_fonts", "system_fonts", "wgpu", "x11", "wayland", "clipboard"]
        check_native_ui_features({"features": features, "uses_default_features": False})
        for invalid in [features[:-1], features+["links"]]:
            with self.assertRaises(RuntimeError):
                check_native_ui_features({"features": invalid, "uses_default_features": False})
        with self.assertRaises(RuntimeError):
            check_native_ui_features({"features": features, "uses_default_features": True})

    def test_manual_coverage_requires_events_as_well_as_claims(self):
        r = fixture()
        r["native_event_observations"]["text_game_input_leaks"] = 0
        validate(r, "native_user_session")
        a = assess(r)
        self.assertTrue(a["observed"]["keyboard_editor_isolation"])
        self.assertTrue(all(a["user_confirmed"].values()))
        self.assertFalse(a["product_acceptance_passed"])
        r["native_event_observations"]["ime_preedit"] = 0
        self.assertFalse(assess(r)["user_confirmed"]["chinese_ime"])
        r["native_event_observations"]["drop_accepted"] = 0
        self.assertFalse(assess(r)["user_confirmed"]["drop_thumbnail"])
        r["native_event_observations"]["text_zx_while_playing"] = 0
        self.assertFalse(assess(r)["observed"]["keyboard_editor_isolation"])

    def test_smoke_or_synthetic_input_never_certifies_manual_acceptance(self):
        r = fixture()
        r["source"] = "native_smoke_no_manual_input"
        with self.assertRaises(RuntimeError):
            validate(r, r["source"])
        r["native_event_observations"] = dict.fromkeys(EVENTS, 0)
        r["human_confirmations"] = dict.fromkeys(CLAIMS, False)
        validate(r, r["source"])
        self.assertFalse(any(assess(r)["observed"].values()))
        self.assertFalse(any(assess(r)["user_confirmed"].values()))
        r["source"] = "synthetic"
        with self.assertRaises(RuntimeError):
            validate(r, r["source"])

    def test_ui_zoom_does_not_prove_native_dpi_transition(self):
        r = fixture()
        r["scale_observations"].append({"native_pixels_per_point": 1.25, "effective_pixels_per_point": 1.875, "ui_zoom": 1.5})
        validate(r, r["source"])
        self.assertFalse(assess(r)["observed"]["native_dpi_transition"])
        r["scale_observations"].append({"native_pixels_per_point": 2.0, "effective_pixels_per_point": 3.0, "ui_zoom": 1.5})
        validate(r, r["source"])
        self.assertTrue(assess(r)["observed"]["native_dpi_transition"])
        r["scale_observations"][-1]["effective_pixels_per_point"] = 1.5
        with self.assertRaises(RuntimeError):
            validate(r, r["source"])

    def test_text_payload_extra_fields_missing_events_and_source_mismatch_rejected(self):
        for key, value in [("raw_text", "accidental paste"), ("foreign_path", "foreign.png"), ("input_text_persisted", True)]:
            r = fixture(); r[key] = value
            with self.subTest(key=key), self.assertRaises(RuntimeError):
                validate(r, r["source"])
        r = fixture(); del r["native_event_observations"]["ime_commit"]
        with self.assertRaises(RuntimeError):
            validate(r, r["source"])
        with self.assertRaises(RuntimeError):
            validate(fixture(), "native_smoke_no_manual_input")

    def test_input_leak_and_unavailable_environment_cannot_pass(self):
        r = fixture()
        self.assertTrue(assess(r)["input_leak_detected"])
        self.assertFalse(assess(r)["observed"]["keyboard_editor_isolation"])
        r["unavailable"]["japanese_ime"] = True
        with self.assertRaises(RuntimeError):
            validate(r, r["source"])
        r["human_confirmations"]["japanese_ime"] = False
        validate(r, r["source"])
        self.assertFalse(assess(r)["user_confirmed"]["japanese_ime"])
        self.assertTrue(assess(r)["unavailable"]["japanese_ime"])


if __name__ == "__main__":
    unittest.main()
