from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar, Iterable, Optional

DESCRIPTOR: _descriptor.FileDescriptor

class AmznScrapingRequest(_message.Message):
    __slots__ = ["product_codes", "request_timestamp"]
    PRODUCT_CODES_FIELD_NUMBER: ClassVar[int]
    REQUEST_TIMESTAMP_FIELD_NUMBER: ClassVar[int]
    product_codes: _containers.RepeatedScalarFieldContainer[str]
    request_timestamp: int
    def __init__(self, request_timestamp: Optional[int] = ..., product_codes: Optional[Iterable[str]] = ...) -> None: ...

class TestMessage(_message.Message):
    __slots__ = ["content"]
    CONTENT_FIELD_NUMBER: ClassVar[int]
    content: str
    def __init__(self, content: Optional[str] = ...) -> None: ...
