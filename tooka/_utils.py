import re

CRON_FIELD_RANGES = {
    "minute": (0, 59),
    "hour": (0, 23),
    "day": (1, 31),
    "month": (1, 12),
    "day_of_week": (0, 6),
}

def is_valid_cron_field(field: str, name: str) -> bool:
    """Validate a single cron field."""
    valid_pattern = re.compile(r'^(\*|\d+|\d+-\d+|\*/\d+|\d+(,\d+)*|\d+-\d+/\d+)$')
    for part in field.split(','):
        if not valid_pattern.match(part):
            return False
        # Extract numbers and ranges for range-checking
        try:
            if part == '*':
                continue
            elif part.startswith('*/'):
                step = int(part[2:])
                if step <= 0:
                    return False
            elif '-' in part and '/' in part:
                range_part, step = part.split('/')
                start, end = map(int, range_part.split('-'))
                if not (CRON_FIELD_RANGES[name][0] <= start <= end <= CRON_FIELD_RANGES[name][1]):
                    return False
                if int(step) <= 0:
                    return False
            elif '-' in part:
                start, end = map(int, part.split('-'))
                if not (CRON_FIELD_RANGES[name][0] <= start <= end <= CRON_FIELD_RANGES[name][1]):
                    return False
            else:
                num = int(part)
                if not (CRON_FIELD_RANGES[name][0] <= num <= CRON_FIELD_RANGES[name][1]):
                    return False
        except ValueError:
            return False
    return True

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
        "day_of_week": day_of_week
    }
