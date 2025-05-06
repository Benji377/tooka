"""Utility collection. Functions used across all the project"""

import re

CRON_FIELD_RANGES = {
    "minute": (0, 59),
    "hour": (0, 23),
    "day": (1, 31),
    "month": (1, 12),
    "day_of_week": (0, 6),
}

CRON_PATTERN = re.compile(r'^(\*|\d+|\d+-\d+|\*/\d+|\d+(,\d+)*|\d+-\d+/\d+)$')

def _validate_cron_part(part: str, min_val: int, max_val: int) -> bool:
    """Helper to validate a single cron field part."""
    try:
        if part == '*':
            return True
        if part.startswith('*/'):
            return int(part[2:]) > 0
        if '-' in part and '/' in part:
            range_part, step = part.split('/')
            start, end = map(int, range_part.split('-'))
            return min_val <= start <= end <= max_val and int(step) > 0
        if '-' in part:
            start, end = map(int, part.split('-'))
            return min_val <= start <= end <= max_val
        return min_val <= int(part) <= max_val
    except ValueError:
        return False

def is_valid_cron_field(field: str, name: str) -> bool:
    """Validate a single cron field."""
    min_val, max_val = CRON_FIELD_RANGES[name]
    return all(
        CRON_PATTERN.match(part) and _validate_cron_part(part, min_val, max_val)
        for part in field.split(',')
    )

def is_valid_cron_expression(expr: str) -> bool:
    """Validates full 5-part cron expression with stricter rules."""
    parts = expr.strip().split()
    if len(parts) != 5:
        return False

    keys = ["minute", "hour", "day", "month", "day_of_week"]
    return all(is_valid_cron_field(field, name) for field, name in zip(parts, keys))

def parse_cron_expression(expr: str) -> dict:
    """Parses validated cron expression into a dictionary."""
    if not is_valid_cron_expression(expr):
        raise ValueError(f"Invalid cron expression: '{expr}'")

    minute, hour, day, month, day_of_week = expr.strip().split()
    return {
        "minute": minute,
        "hour": hour,
        "day": day,
        "month": month,
        "day_of_week": day_of_week,
    }
