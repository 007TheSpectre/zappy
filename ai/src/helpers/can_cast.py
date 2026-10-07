def can_cast(src_t, dest_t, obj):
    try:
        dest_t(src_t(obj))
        return True
    except Exception:
        return False
