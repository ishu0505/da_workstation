"""DEMO FIXTURE tests. Run: python3 -m unittest -v (two tests fail on purpose: one planted bug)."""

import unittest

from pricing import apply_discount, total


class PricingTest(unittest.TestCase):
    def test_no_discount(self):
        self.assertEqual(apply_discount(80.0, 0), 80.0)

    def test_ten_percent(self):
        self.assertEqual(apply_discount(80.0, 10), 72.0)

    def test_total(self):
        self.assertEqual(total([10.0, 20.0], 50), 15.0)

    def test_rejects_bad_percent(self):
        with self.assertRaises(ValueError):
            apply_discount(10.0, 120)


if __name__ == "__main__":
    unittest.main()
