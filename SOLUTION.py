from typing import List

def generate_error_docs() -> str:
    variants = [
        (1, "MissingRecord", "Raised when `rebind_wallet` or `admin_deactivate` finds no prior state."),
        (2, "Unauthorized", "Raised by `require_caller` in `payments::settle_intent`."),
        (3, "AccountLocked", "Raised when a locked account attempts a transaction."),
        (4, "DuplicateId", "Raised when an ID is registered twice."),
        (5, "PendingState", "Raised when state is expected ready."),
        (6, "TooManySigs", "Raised when too many signatures provided."),
        (7, "TooManyAccounts", "Raised for excessive account loading."),
        (8, "InvalidProvider", "Raised for malformed provider data."),
        (9, "WalletAlreadyBound", "Fires whenever any binding exists, blocking `bind_wallet` outright."),
        (10, "ReentrantCall", "Raised by `NonReentrantGuard::acquire`, used by `settle_intent` and `cancel_intent`."),
    ]

    header = "| # | Code | Name | Description |"
    # Separator: 20 dashes for Name col to accommodate "WalletAlreadyBound" (18 chars)
    separator = "|---|-----|------|--------------------|--------------------------|"
    
    rows = []
    for code, name, desc in variants:
        rows.append(f"| {code} | {name} | {desc} |")

    return "\n".join([header, separator, *rows])

if __name__ == "__main__":
    print(generate_error_docs())