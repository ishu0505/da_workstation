"""DEMO FIXTURE for DA Workstation: a small module with one planted bug.

`apply_discount` should take a percentage off the price. The bug divides by 10
instead of 100, so two tests in test_pricing.py fail until it is fixed.
"""


def apply_discount(price: float, percent: float) -> float:
    if not 0 <= percent <= 100:
        raise ValueError("percent must be between 0 and 100")
    return round(price * (1 - percent / 10), 2)


def total(prices: list[float], percent: float = 0) -> float:
    return round(sum(apply_discount(p, percent) for p in prices), 2)
