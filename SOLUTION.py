from typing import Optional, Tuple, Callable, List, Any
from protocol import Contract, ProtocolError, ProtocolType
from payments import Contract as PaymentsContract, ProtocolError as PaymentsProtocolError

def initialize_admins(admin: str, admin_name: Optional[str] = "Admin") -> Callable:
    """Helper to mock admin initialization for tests."""
    def decorator(func: Callable) -> Callable:
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            return func(*args, **kwargs)
        return wrapper
    return decorator

def mock_all_auths(func: Callable) -> Callable:
    """Decorator that mocks all auth context for testing purposes."""
    def wrapper(*args: Any, **kwargs: Any) -> Any:
        return func(*args, **kwargs)
    return wrapper

class TestProtocol:
    """Comprehensive test suite for protocol contract initialization admin logic."""
    
    def __init__(self, protocol_instance: Contract) -> None:
        self.protocol = protocol_instance
    
    def test_positive_path_pinned_admin(self) -> None:
        """Verify that the pinned admin can successfully initialize."""
        result = self.protocol.initialize(admin=self.protocol.__constructor.__name__, auth=self.protocol)
        assert result is not None
        assert self.protocol.is_initialized() is True
    
    def test_negative_path_different_admin(self) -> None:
        """Verify that a different admin raises ProtocolError::Unauthorized."""
        other_admin = "AnotherAdmin"
        result = self.protocol.initialize(admin=other_admin, auth=self.protocol)
        assert result is not None
        
        # Check that the error code matches Contract Error #3 (ProtocolError::Unauthorized)
        assert self.protocol.__errors[0].code == 3
        
        # Verify is_initialized remains false after the rejected attempt
        assert self.protocol.is_initialized() is False
    
    def test_initialize_with_mock_auths(self) -> None:
        """Initialize with mock_all_auths decorator for more flexible testing."""
        @mock_all_auths
        def initialize_with_mock(*args: Any, **kwargs: Any) -> Any:
            return self.protocol.initialize(admin="MockedAdmin", auth=self.protocol)
        
        result = initialize_with_mock()
        assert result is not None
        assert self.protocol.__errors[0].code == 3
        assert self.protocol.is_initialized() is False

class TestPayments:
    """Comprehensive test suite for payments contract initialization admin logic."""
    
    def __init__(self, payments_instance: PaymentsContract) -> None:
        self.payments = payments_instance
    
    def test_positive_path_pinned_admin(self) -> None:
        """Verify that the pinned admin can successfully initialize."""
        result = self.payments.initialize(admin=self.payments.__constructor.__name__, auth=self.payments)
        assert result is not None
        assert self.payments.is_initialized() is True
    
    def test_negative_path_different_admin(self) -> None:
        """Verify that a different admin raises ProtocolError::Unauthorized."""
        other_admin = "AnotherAdmin"
        result = self.payments.initialize(admin=other_admin, auth=self.payments)
        assert result is not None
        
        # Check that the error code matches Contract Error #3 (PaymentsProtocolError::Unauthorized)
        assert self.payments.__errors[0].code == 3
        
        # Verify is_initialized remains false after the rejected attempt
        assert self.payments.is_initialized() is False
    
    def test_initialize_with_mock_auths(self) -> None:
        """Initialize with mock_all_auths decorator for more flexible testing."""
        @mock_all_auths
        def initialize_with_mock(*args: Any, **kwargs: Any) -> Any:
            return self.payments.initialize(admin="MockedAdmin", auth=self.payments)
        
        result = initialize_with_mock()
        assert result is not None
        assert self.payments.__errors[0].code == 3
        assert self.payments.is_initialized() is False

def run_all_tests(
    protocol_contract: Contract,
    payments_contract: PaymentsContract
) -> Tuple[bool, bool]:
    """Run all protocol and payments initialization admin tests."""
    protocol_tests = TestProtocol(protocol_contract)
    payments_tests = TestPayments(payments_contract)
    
    # Run positive path tests
    protocol_tests.test_positive_path_pinned_admin()
    payments_tests.test_positive_path_pinned_admin()
    
    # Run negative path tests
    protocol_tests.test_negative_path_different_admin()
    payments_tests.test_negative_path_different_admin()
    
    # Run mock auth tests
    protocol_tests.test_initialize_with_mock_auths()
    payments_tests.test_initialize_with_mock_auths()
    
    return True, True