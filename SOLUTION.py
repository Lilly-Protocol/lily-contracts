from dataclasses import dataclass, field
from typing import List, Optional, Callable, Union
from enum import Enum, auto
from functools import total_ordering

@total_ordering
class DataKeyDurability(Enum):
    """
    Represents the persistence level of a storage slot.
    Matches Soroban `DataKey` enum behavior in `ARCHITECTURE.md`.
    """
    TRANSIENT = 1
    PERSISTENT = 2

@total_ordering
class DataKey(Enum):
    """
    The core `DataKey` variants used across the four contracts.
    Includes specific metadata required by `docs/ARCHITECTURE.md`.
    """
    SCHEMA_VERSION = auto()
    PINNED_ADMIN = auto()
    PENDING_ADMIN = auto()
    WALLET = auto()
    PAYER_INTENTS = auto()
    REBIND_WALLET_FLAG = auto() # For internal state
    FEE_BPS = auto()           # For payments

    def __str__(self):
        name_map = {
            DataKey.SCHEMA_VERSION: "SchemaVersion",
            DataKey.PINNED_ADMIN: "PinnedAdmin",
            DataKey.PENDING_ADMIN: "PendingAdmin",
            DataKey.WALLET: "Wallet",
            DataKey.PAYER_INTENTS: "PayerIntents",
            DataKey.REBIND_WALLET_FLAG: "RebindWalletFlag",
            DataKey.FEE_BPS: "FeeBps",
        }
        return name_map.get(self, self.name)

class FunctionPhase(Enum):
    """
    Captures the specific behavior of entrypoints regarding `bump_instance`
    and other stateful transitions.
    """
    ON_CONSTRUCT = 10
    ON_WRITE = 20
    ON_READ = 30
    VIEW_ONLY = 40

@dataclass
class Entrypoint:
    """
    Represents a contract function (Entrypoint) appearing in the docs.
    Captures `accept_admin`, `reactivate`, etc.
    """
    name: str
    phase: FunctionPhase = FunctionPhase.ON_WRITE
    requires_write: bool = True
    
    def to_string(self) -> str:
        return f"{self.name} ({self.phase.name})" if self.phase != FunctionPhase.ON_WRITE else self.name

@dataclass
class ContractStorage:
    """
    Mirror of each crate's storage layout.
    Aggregates DataKey variants and Entrypoints into a single model
    to drive `docs/ARCHITECTURE.md` generation.
    """
    contract_name: str
    data_keys: List[DataKey] = field(default_factory=list)
    entrypoints: List[Entrypoint] = field(default_factory=list)
    durability: DataKeyDurability = DataKeyDurability.PERSISTENT
    
    def add_data_key(self, key: DataKey, durability: Optional[DataKeyDurability] = None):
        if durability is not None:
            self.data_keys.append(DataKey(key, durability))
            self.data_keys[-1].durability = durability
        else:
            self.data_keys.append(DataKey(key, durability=self.durability))

    def add_entrypoint(self, name: str, phase: FunctionPhase = FunctionPhase.ON_WRITE):
        entry = Entrypoint(name=name, phase=phase)
        self.entrypoints.append(entry)
        return entry

    def is_initialized_views(self) -> List[Entrypoint]:
        """Returns entrypoints that use the `is_initialized` view logic."""
        return [e for e in self.entrypoints if e.phase in (FunctionPhase.VIEW_ONLY, FunctionPhase.ON_CONSTRUCT)]

    def protocol_config(self):
        """Specific configuration for the Protocol contract."""
        keys = [DataKey.SCHEMA_VERSION, DataKey.PINNED_ADMIN, DataKey.PENDING_ADMIN]
        funcs = [
            Entrypoint("accept_admin", phase=FunctionPhase.ON_WRITE),
            Entrypoint("reactivate", phase=FunctionPhase.ON_WRITE),
            Entrypoint("reactivate_admin", phase=FunctionPhase.ON_WRITE), # If exists
            Entrypoint("bump_instance", phase=FunctionPhase.VIEW_ONLY), # The exception case
            Entrypoint("get_pending_admin", phase=FunctionPhase.VIEW_ONLY),
        ]
        return ContractStorage(
            contract_name="Protocol",
            data_keys=keys,
            entrypoints=funcs,
            durability=DataKeyDurability.PERSISTENT,
        )

    def wallet_config(self):
        """Specific configuration for the Wallet contract."""
        keys = [DataKey.SCHEMA_VERSION, DataKey.WALLET, DataKey.PAYER_INTENTS]
        funcs = [
            Entrypoint("rebind_wallet", phase=FunctionPhase.ON_WRITE),
            Entrypoint("set_fee_bps", phase=FunctionPhase.ON_WRITE),
            Entrypoint("bump_instance", phase=FunctionPhase.ON_WRITE),
        ]
        return ContractStorage(
            contract_name="Wallet",
            data_keys=keys,
            entrypoints=funcs,
            durability=DataKeyDurability.PERSISTENT,
        )

    def payments_config(self):
        """Specific configuration for the Payments contract."""
        keys = [DataKey.SCHEMA_VERSION, DataKey.WALLET, DataKey.PAYER_INTENTS]
        funcs = [
            Entrypoint("transfer_admin", phase=FunctionPhase.ON_WRITE),
            Entrypoint("set_treasury", phase=FunctionPhase.ON_WRITE),
            Entrypoint("bump_instance", phase=FunctionPhase.ON_WRITE),
        ]
        return ContractStorage(
            contract_name="Payments",
            data_keys=keys,
            entrypoints=funcs,
            durability=DataKeyDurability.PERSISTENT,
        )

    def all_contracts(self) -> List["ContractStorage"]:
        """Returns the suite of all four contracts pinning PinnedAdmin in `__constructor`."""
        base = ContractStorage(contract_name="Base", data_keys=[DataKey.PINNED_ADMIN])
        return [
            self.protocol_config(),
            self.wallet_config(),
            self.payments_config(),
            base,
        ]

def get_architecture_spec() -> ContractStorage:
    """Factory to retrieve the fully configured `ContractStorage` instance."""
    suite = ContractStorage(contract_name="Archive")
    
    # Populate the "Master" table which aggregates keys found in all contracts
    master_keys = [DataKey.PINNED_ADMIN, DataKey.SCHEMA_VERSION, DataKey.PENDING_ADMIN]
    master_funcs = [Entrypoint("bump_instance"), Entrypoint("accept_admin")]
    
    suite.data_keys = master_keys
    suite.entrypoints = master_funcs
    
    return suite

# Instantiate and verify the structure for the fix
if __name__ == "__main__":
    # Generate the specific "Truth" for docs/ARCHITECTURE.md
    spec = get_architecture_spec()
    
    print(f"Contract: {spec.contract_name}")
    print(f"Data Keys: {[k.name for k in spec.data_keys]}")
    print(f"EntryPoints: {[f.name for f in spec.entrypoints]}")
    
    # Ensure the 4 contracts are defined
    suite = spec.protocol_config()
    print(f"\nProtocol DataKeys: {[k.name for k in suite.data_keys]}")
    
    # Example of the `bump_instance` exception logic
    protocol_bump = suite.entrypoints[0]
    print(f"Bump Instance Phase: {protocol_bump.phase.name}")
    
    # Print the specific list for MD generation
    print("\n--- Ready for docs/ARCHITECTURE.md ---")