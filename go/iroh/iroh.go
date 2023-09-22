package iroh

// #include <irohFFI.h>
import "C"

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"math"
	"runtime"
	"sync"
	"sync/atomic"
	"unsafe"
)

type RustBuffer = C.RustBuffer

type RustBufferI interface {
	AsReader() *bytes.Reader
	Free()
	ToGoBytes() []byte
	Data() unsafe.Pointer
	Len() int
	Capacity() int
}

func RustBufferFromForeign(b RustBufferI) RustBuffer {
	return RustBuffer{
		capacity: C.int(b.Capacity()),
		len:      C.int(b.Len()),
		data:     (*C.uchar)(b.Data()),
	}
}

func (cb RustBuffer) Capacity() int {
	return int(cb.capacity)
}

func (cb RustBuffer) Len() int {
	return int(cb.len)
}

func (cb RustBuffer) Data() unsafe.Pointer {
	return unsafe.Pointer(cb.data)
}

func (cb RustBuffer) AsReader() *bytes.Reader {
	b := unsafe.Slice((*byte)(cb.data), C.int(cb.len))
	return bytes.NewReader(b)
}

func (cb RustBuffer) Free() {
	rustCall(func(status *C.RustCallStatus) bool {
		C.ffi_iroh_rustbuffer_free(cb, status)
		return false
	})
}

func (cb RustBuffer) ToGoBytes() []byte {
	return C.GoBytes(unsafe.Pointer(cb.data), C.int(cb.len))
}

func stringToRustBuffer(str string) RustBuffer {
	return bytesToRustBuffer([]byte(str))
}

func bytesToRustBuffer(b []byte) RustBuffer {
	if len(b) == 0 {
		return RustBuffer{}
	}
	// We can pass the pointer along here, as it is pinned
	// for the duration of this call
	foreign := C.ForeignBytes{
		len:  C.int(len(b)),
		data: (*C.uchar)(unsafe.Pointer(&b[0])),
	}

	return rustCall(func(status *C.RustCallStatus) RustBuffer {
		return C.ffi_iroh_rustbuffer_from_bytes(foreign, status)
	})
}

type BufLifter[GoType any] interface {
	Lift(value RustBufferI) GoType
}

type BufLowerer[GoType any] interface {
	Lower(value GoType) RustBuffer
}

type FfiConverter[GoType any, FfiType any] interface {
	Lift(value FfiType) GoType
	Lower(value GoType) FfiType
}

type BufReader[GoType any] interface {
	Read(reader io.Reader) GoType
}

type BufWriter[GoType any] interface {
	Write(writer io.Writer, value GoType)
}

type FfiRustBufConverter[GoType any, FfiType any] interface {
	FfiConverter[GoType, FfiType]
	BufReader[GoType]
}

func LowerIntoRustBuffer[GoType any](bufWriter BufWriter[GoType], value GoType) RustBuffer {
	// This might be not the most efficient way but it does not require knowing allocation size
	// beforehand
	var buffer bytes.Buffer
	bufWriter.Write(&buffer, value)

	bytes, err := io.ReadAll(&buffer)
	if err != nil {
		panic(fmt.Errorf("reading written data: %w", err))
	}
	return bytesToRustBuffer(bytes)
}

func LiftFromRustBuffer[GoType any](bufReader BufReader[GoType], rbuf RustBufferI) GoType {
	defer rbuf.Free()
	reader := rbuf.AsReader()
	item := bufReader.Read(reader)
	if reader.Len() > 0 {
		// TODO: Remove this
		leftover, _ := io.ReadAll(reader)
		panic(fmt.Errorf("Junk remaining in buffer after lifting: %s", string(leftover)))
	}
	return item
}

func rustCallWithError[U any](converter BufLifter[error], callback func(*C.RustCallStatus) U) (U, error) {
	var status C.RustCallStatus
	returnValue := callback(&status)
	err := checkCallStatus(converter, status)

	return returnValue, err
}

func checkCallStatus(converter BufLifter[error], status C.RustCallStatus) error {
	switch status.code {
	case 0:
		return nil
	case 1:
		return converter.Lift(status.errorBuf)
	case 2:
		// when the rust code sees a panic, it tries to construct a rustbuffer
		// with the message.  but if that code panics, then it just sends back
		// an empty buffer.
		if status.errorBuf.len > 0 {
			panic(fmt.Errorf("%s", FfiConverterStringINSTANCE.Lift(status.errorBuf)))
		} else {
			panic(fmt.Errorf("Rust panicked while handling Rust panic"))
		}
	default:
		return fmt.Errorf("unknown status code: %d", status.code)
	}
}

func checkCallStatusUnknown(status C.RustCallStatus) error {
	switch status.code {
	case 0:
		return nil
	case 1:
		panic(fmt.Errorf("function not returning an error returned an error"))
	case 2:
		// when the rust code sees a panic, it tries to construct a rustbuffer
		// with the message.  but if that code panics, then it just sends back
		// an empty buffer.
		if status.errorBuf.len > 0 {
			panic(fmt.Errorf("%s", FfiConverterStringINSTANCE.Lift(status.errorBuf)))
		} else {
			panic(fmt.Errorf("Rust panicked while handling Rust panic"))
		}
	default:
		return fmt.Errorf("unknown status code: %d", status.code)
	}
}

func rustCall[U any](callback func(*C.RustCallStatus) U) U {
	returnValue, err := rustCallWithError(nil, callback)
	if err != nil {
		panic(err)
	}
	return returnValue
}

func writeInt8(writer io.Writer, value int8) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint8(writer io.Writer, value uint8) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeInt16(writer io.Writer, value int16) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint16(writer io.Writer, value uint16) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeInt32(writer io.Writer, value int32) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint32(writer io.Writer, value uint32) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeInt64(writer io.Writer, value int64) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint64(writer io.Writer, value uint64) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeFloat32(writer io.Writer, value float32) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeFloat64(writer io.Writer, value float64) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func readInt8(reader io.Reader) int8 {
	var result int8
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint8(reader io.Reader) uint8 {
	var result uint8
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readInt16(reader io.Reader) int16 {
	var result int16
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint16(reader io.Reader) uint16 {
	var result uint16
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readInt32(reader io.Reader) int32 {
	var result int32
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint32(reader io.Reader) uint32 {
	var result uint32
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readInt64(reader io.Reader) int64 {
	var result int64
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint64(reader io.Reader) uint64 {
	var result uint64
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readFloat32(reader io.Reader) float32 {
	var result float32
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readFloat64(reader io.Reader) float64 {
	var result float64
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func init() {

	(&FfiConverterCallbackInterfaceSubscribeCallback{}).register()
}

type FfiConverterUint8 struct{}

var FfiConverterUint8INSTANCE = FfiConverterUint8{}

func (FfiConverterUint8) Lower(value uint8) C.uint8_t {
	return C.uint8_t(value)
}

func (FfiConverterUint8) Write(writer io.Writer, value uint8) {
	writeUint8(writer, value)
}

func (FfiConverterUint8) Lift(value C.uint8_t) uint8 {
	return uint8(value)
}

func (FfiConverterUint8) Read(reader io.Reader) uint8 {
	return readUint8(reader)
}

type FfiDestroyerUint8 struct{}

func (FfiDestroyerUint8) Destroy(_ uint8) {}

type FfiConverterUint16 struct{}

var FfiConverterUint16INSTANCE = FfiConverterUint16{}

func (FfiConverterUint16) Lower(value uint16) C.uint16_t {
	return C.uint16_t(value)
}

func (FfiConverterUint16) Write(writer io.Writer, value uint16) {
	writeUint16(writer, value)
}

func (FfiConverterUint16) Lift(value C.uint16_t) uint16 {
	return uint16(value)
}

func (FfiConverterUint16) Read(reader io.Reader) uint16 {
	return readUint16(reader)
}

type FfiDestroyerUint16 struct{}

func (FfiDestroyerUint16) Destroy(_ uint16) {}

type FfiConverterUint64 struct{}

var FfiConverterUint64INSTANCE = FfiConverterUint64{}

func (FfiConverterUint64) Lower(value uint64) C.uint64_t {
	return C.uint64_t(value)
}

func (FfiConverterUint64) Write(writer io.Writer, value uint64) {
	writeUint64(writer, value)
}

func (FfiConverterUint64) Lift(value C.uint64_t) uint64 {
	return uint64(value)
}

func (FfiConverterUint64) Read(reader io.Reader) uint64 {
	return readUint64(reader)
}

type FfiDestroyerUint64 struct{}

func (FfiDestroyerUint64) Destroy(_ uint64) {}

type FfiConverterFloat64 struct{}

var FfiConverterFloat64INSTANCE = FfiConverterFloat64{}

func (FfiConverterFloat64) Lower(value float64) C.double {
	return C.double(value)
}

func (FfiConverterFloat64) Write(writer io.Writer, value float64) {
	writeFloat64(writer, value)
}

func (FfiConverterFloat64) Lift(value C.double) float64 {
	return float64(value)
}

func (FfiConverterFloat64) Read(reader io.Reader) float64 {
	return readFloat64(reader)
}

type FfiDestroyerFloat64 struct{}

func (FfiDestroyerFloat64) Destroy(_ float64) {}

type FfiConverterBool struct{}

var FfiConverterBoolINSTANCE = FfiConverterBool{}

func (FfiConverterBool) Lower(value bool) C.int8_t {
	if value {
		return C.int8_t(1)
	}
	return C.int8_t(0)
}

func (FfiConverterBool) Write(writer io.Writer, value bool) {
	if value {
		writeInt8(writer, 1)
	} else {
		writeInt8(writer, 0)
	}
}

func (FfiConverterBool) Lift(value C.int8_t) bool {
	return value != 0
}

func (FfiConverterBool) Read(reader io.Reader) bool {
	return readInt8(reader) != 0
}

type FfiDestroyerBool struct{}

func (FfiDestroyerBool) Destroy(_ bool) {}

type FfiConverterString struct{}

var FfiConverterStringINSTANCE = FfiConverterString{}

func (FfiConverterString) Lift(rb RustBufferI) string {
	defer rb.Free()
	reader := rb.AsReader()
	b, err := io.ReadAll(reader)
	if err != nil {
		panic(fmt.Errorf("reading reader: %w", err))
	}
	return string(b)
}

func (FfiConverterString) Read(reader io.Reader) string {
	length := readInt32(reader)
	buffer := make([]byte, length)
	read_length, err := reader.Read(buffer)
	if err != nil {
		panic(err)
	}
	if read_length != int(length) {
		panic(fmt.Errorf("bad read length when reading string, expected %d, read %d", length, read_length))
	}
	return string(buffer)
}

func (FfiConverterString) Lower(value string) RustBuffer {
	return stringToRustBuffer(value)
}

func (FfiConverterString) Write(writer io.Writer, value string) {
	if len(value) > math.MaxInt32 {
		panic("String is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	write_length, err := io.WriteString(writer, value)
	if err != nil {
		panic(err)
	}
	if write_length != len(value) {
		panic(fmt.Errorf("bad write length when writing string, expected %d, written %d", len(value), write_length))
	}
}

type FfiDestroyerString struct{}

func (FfiDestroyerString) Destroy(_ string) {}

type FfiConverterBytes struct{}

var FfiConverterBytesINSTANCE = FfiConverterBytes{}

func (c FfiConverterBytes) Lower(value []byte) RustBuffer {
	return LowerIntoRustBuffer[[]byte](c, value)
}

func (c FfiConverterBytes) Write(writer io.Writer, value []byte) {
	if len(value) > math.MaxInt32 {
		panic("[]byte is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	write_length, err := writer.Write(value)
	if err != nil {
		panic(err)
	}
	if write_length != len(value) {
		panic(fmt.Errorf("bad write length when writing string, expected %d, written %d", len(value), write_length))
	}
}

func (c FfiConverterBytes) Lift(rb RustBufferI) []byte {
	return LiftFromRustBuffer[[]byte](c, rb)
}

func (c FfiConverterBytes) Read(reader io.Reader) []byte {
	length := readInt32(reader)
	buffer := make([]byte, length)
	read_length, err := reader.Read(buffer)
	if err != nil {
		panic(err)
	}
	if read_length != int(length) {
		panic(fmt.Errorf("bad read length when reading []byte, expected %d, read %d", length, read_length))
	}
	return buffer
}

type FfiDestroyerBytes struct{}

func (FfiDestroyerBytes) Destroy(_ []byte) {}

// Below is an implementation of synchronization requirements outlined in the link.
// https://github.com/mozilla/uniffi-rs/blob/0dc031132d9493ca812c3af6e7dd60ad2ea95bf0/uniffi_bindgen/src/bindings/kotlin/templates/ObjectRuntime.kt#L31

type FfiObject struct {
	pointer      unsafe.Pointer
	callCounter  atomic.Int64
	freeFunction func(unsafe.Pointer, *C.RustCallStatus)
	destroyed    atomic.Bool
}

func newFfiObject(pointer unsafe.Pointer, freeFunction func(unsafe.Pointer, *C.RustCallStatus)) FfiObject {
	return FfiObject{
		pointer:      pointer,
		freeFunction: freeFunction,
	}
}

func (ffiObject *FfiObject) incrementPointer(debugName string) unsafe.Pointer {
	for {
		counter := ffiObject.callCounter.Load()
		if counter <= -1 {
			panic(fmt.Errorf("%v object has already been destroyed", debugName))
		}
		if counter == math.MaxInt64 {
			panic(fmt.Errorf("%v object call counter would overflow", debugName))
		}
		if ffiObject.callCounter.CompareAndSwap(counter, counter+1) {
			break
		}
	}

	return ffiObject.pointer
}

func (ffiObject *FfiObject) decrementPointer() {
	if ffiObject.callCounter.Add(-1) == -1 {
		ffiObject.freeRustArcPtr()
	}
}

func (ffiObject *FfiObject) destroy() {
	if ffiObject.destroyed.CompareAndSwap(false, true) {
		if ffiObject.callCounter.Add(-1) == -1 {
			ffiObject.freeRustArcPtr()
		}
	}
}

func (ffiObject *FfiObject) freeRustArcPtr() {
	rustCall(func(status *C.RustCallStatus) int32 {
		ffiObject.freeFunction(ffiObject.pointer, status)
		return 0
	})
}

type AuthorId struct {
	ffiObject FfiObject
}

func (_self *AuthorId) ToString() string {
	_pointer := _self.ffiObject.incrementPointer("*AuthorId")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_authorid_to_string(
			_pointer, _uniffiStatus)
	}))

}

func (object *AuthorId) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterAuthorId struct{}

var FfiConverterAuthorIdINSTANCE = FfiConverterAuthorId{}

func (c FfiConverterAuthorId) Lift(pointer unsafe.Pointer) *AuthorId {
	result := &AuthorId{
		newFfiObject(
			pointer,
			func(pointer unsafe.Pointer, status *C.RustCallStatus) {
				C.uniffi_iroh_fn_free_authorid(pointer, status)
			}),
	}
	runtime.SetFinalizer(result, (*AuthorId).Destroy)
	return result
}

func (c FfiConverterAuthorId) Read(reader io.Reader) *AuthorId {
	return c.Lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterAuthorId) Lower(value *AuthorId) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*AuthorId")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterAuthorId) Write(writer io.Writer, value *AuthorId) {
	writeUint64(writer, uint64(uintptr(c.Lower(value))))
}

type FfiDestroyerAuthorId struct{}

func (_ FfiDestroyerAuthorId) Destroy(value *AuthorId) {
	value.Destroy()
}

type Doc struct {
	ffiObject FfiObject
}

func (_self *Doc) All() ([]*Entry, error) {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_doc_all(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue []*Entry
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterSequenceEntryINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *Doc) GetContentBytes(
	hash *Hash) ([]byte, error) {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_doc_get_content_bytes(
			_pointer, FfiConverterHashINSTANCE.Lower(hash), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue []byte
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterBytesINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *Doc) Id() string {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_doc_id(
			_pointer, _uniffiStatus)
	}))

}

func (_self *Doc) SetBytes(
	author *AuthorId,
	key []byte,
	value []byte) (*Hash, error) {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_doc_set_bytes(
			_pointer, FfiConverterAuthorIdINSTANCE.Lower(author), FfiConverterBytesINSTANCE.Lower(key), FfiConverterBytesINSTANCE.Lower(value), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *Hash
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterHashINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *Doc) ShareRead() (*DocTicket, error) {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_doc_share_read(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *DocTicket
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDocTicketINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *Doc) ShareWrite() (*DocTicket, error) {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_doc_share_write(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *DocTicket
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDocTicketINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *Doc) Status() (LiveStatus, error) {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_doc_status(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue LiveStatus
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterTypeLiveStatusINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *Doc) StopSync() error {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	_, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_iroh_fn_method_doc_stop_sync(
			_pointer, _uniffiStatus)
		return false
	})
	return _uniffiErr

}

func (_self *Doc) Subscribe(
	cb SubscribeCallback) error {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	_, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_iroh_fn_method_doc_subscribe(
			_pointer, FfiConverterCallbackInterfaceSubscribeCallbackINSTANCE.Lower(cb), _uniffiStatus)
		return false
	})
	return _uniffiErr

}

func (object *Doc) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterDoc struct{}

var FfiConverterDocINSTANCE = FfiConverterDoc{}

func (c FfiConverterDoc) Lift(pointer unsafe.Pointer) *Doc {
	result := &Doc{
		newFfiObject(
			pointer,
			func(pointer unsafe.Pointer, status *C.RustCallStatus) {
				C.uniffi_iroh_fn_free_doc(pointer, status)
			}),
	}
	runtime.SetFinalizer(result, (*Doc).Destroy)
	return result
}

func (c FfiConverterDoc) Read(reader io.Reader) *Doc {
	return c.Lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterDoc) Lower(value *Doc) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*Doc")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterDoc) Write(writer io.Writer, value *Doc) {
	writeUint64(writer, uint64(uintptr(c.Lower(value))))
}

type FfiDestroyerDoc struct{}

func (_ FfiDestroyerDoc) Destroy(value *Doc) {
	value.Destroy()
}

type DocTicket struct {
	ffiObject FfiObject
}

func DocTicketFromString(
	content string) (*DocTicket, error) {

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_constructor_docticket_from_string(FfiConverterStringINSTANCE.Lower(content), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *DocTicket
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDocTicketINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *DocTicket) ToString() string {
	_pointer := _self.ffiObject.incrementPointer("*DocTicket")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_docticket_to_string(
			_pointer, _uniffiStatus)
	}))

}

func (object *DocTicket) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterDocTicket struct{}

var FfiConverterDocTicketINSTANCE = FfiConverterDocTicket{}

func (c FfiConverterDocTicket) Lift(pointer unsafe.Pointer) *DocTicket {
	result := &DocTicket{
		newFfiObject(
			pointer,
			func(pointer unsafe.Pointer, status *C.RustCallStatus) {
				C.uniffi_iroh_fn_free_docticket(pointer, status)
			}),
	}
	runtime.SetFinalizer(result, (*DocTicket).Destroy)
	return result
}

func (c FfiConverterDocTicket) Read(reader io.Reader) *DocTicket {
	return c.Lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterDocTicket) Lower(value *DocTicket) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*DocTicket")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterDocTicket) Write(writer io.Writer, value *DocTicket) {
	writeUint64(writer, uint64(uintptr(c.Lower(value))))
}

type FfiDestroyerDocTicket struct{}

func (_ FfiDestroyerDocTicket) Destroy(value *DocTicket) {
	value.Destroy()
}

type Entry struct {
	ffiObject FfiObject
}

func (_self *Entry) Author() *AuthorId {
	_pointer := _self.ffiObject.incrementPointer("*Entry")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterAuthorIdINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_entry_author(
			_pointer, _uniffiStatus)
	}))

}

func (_self *Entry) Hash() *Hash {
	_pointer := _self.ffiObject.incrementPointer("*Entry")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterHashINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_entry_hash(
			_pointer, _uniffiStatus)
	}))

}

func (_self *Entry) Key() []byte {
	_pointer := _self.ffiObject.incrementPointer("*Entry")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterBytesINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_entry_key(
			_pointer, _uniffiStatus)
	}))

}

func (object *Entry) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterEntry struct{}

var FfiConverterEntryINSTANCE = FfiConverterEntry{}

func (c FfiConverterEntry) Lift(pointer unsafe.Pointer) *Entry {
	result := &Entry{
		newFfiObject(
			pointer,
			func(pointer unsafe.Pointer, status *C.RustCallStatus) {
				C.uniffi_iroh_fn_free_entry(pointer, status)
			}),
	}
	runtime.SetFinalizer(result, (*Entry).Destroy)
	return result
}

func (c FfiConverterEntry) Read(reader io.Reader) *Entry {
	return c.Lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterEntry) Lower(value *Entry) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*Entry")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterEntry) Write(writer io.Writer, value *Entry) {
	writeUint64(writer, uint64(uintptr(c.Lower(value))))
}

type FfiDestroyerEntry struct{}

func (_ FfiDestroyerEntry) Destroy(value *Entry) {
	value.Destroy()
}

type Hash struct {
	ffiObject FfiObject
}

func (_self *Hash) ToBytes() []byte {
	_pointer := _self.ffiObject.incrementPointer("*Hash")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterBytesINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_hash_to_bytes(
			_pointer, _uniffiStatus)
	}))

}

func (_self *Hash) ToString() string {
	_pointer := _self.ffiObject.incrementPointer("*Hash")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_hash_to_string(
			_pointer, _uniffiStatus)
	}))

}

func (object *Hash) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterHash struct{}

var FfiConverterHashINSTANCE = FfiConverterHash{}

func (c FfiConverterHash) Lift(pointer unsafe.Pointer) *Hash {
	result := &Hash{
		newFfiObject(
			pointer,
			func(pointer unsafe.Pointer, status *C.RustCallStatus) {
				C.uniffi_iroh_fn_free_hash(pointer, status)
			}),
	}
	runtime.SetFinalizer(result, (*Hash).Destroy)
	return result
}

func (c FfiConverterHash) Read(reader io.Reader) *Hash {
	return c.Lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterHash) Lower(value *Hash) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*Hash")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterHash) Write(writer io.Writer, value *Hash) {
	writeUint64(writer, uint64(uintptr(c.Lower(value))))
}

type FfiDestroyerHash struct{}

func (_ FfiDestroyerHash) Destroy(value *Hash) {
	value.Destroy()
}

type IrohNode struct {
	ffiObject FfiObject
}

func NewIrohNode() (*IrohNode, error) {

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_constructor_irohnode_new(_uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *IrohNode
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterIrohNodeINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *IrohNode) ConnectionInfo(
	nodeId *PublicKey) (*ConnectionInfo, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_irohnode_connection_info(
			_pointer, FfiConverterPublicKeyINSTANCE.Lower(nodeId), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *ConnectionInfo
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterOptionalTypeConnectionInfoINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *IrohNode) Connections() ([]ConnectionInfo, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_irohnode_connections(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue []ConnectionInfo
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterSequenceTypeConnectionInfoINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *IrohNode) CreateAuthor() (*AuthorId, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_irohnode_create_author(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *AuthorId
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterAuthorIdINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *IrohNode) CreateDoc() (*Doc, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_irohnode_create_doc(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *Doc
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDocINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *IrohNode) ImportDoc(
	ticket *DocTicket) (*Doc, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_irohnode_import_doc(
			_pointer, FfiConverterDocTicketINSTANCE.Lower(ticket), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *Doc
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDocINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *IrohNode) ListAuthors() ([]*AuthorId, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_irohnode_list_authors(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue []*AuthorId
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterSequenceAuthorIdINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (_self *IrohNode) PeerId() string {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_irohnode_peer_id(
			_pointer, _uniffiStatus)
	}))

}

func (_self *IrohNode) Stats() (map[string]CounterStats, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_irohnode_stats(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue map[string]CounterStats
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterMapStringTypeCounterStatsINSTANCE.Lift(_uniffiRV), _uniffiErr
	}

}

func (object *IrohNode) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterIrohNode struct{}

var FfiConverterIrohNodeINSTANCE = FfiConverterIrohNode{}

func (c FfiConverterIrohNode) Lift(pointer unsafe.Pointer) *IrohNode {
	result := &IrohNode{
		newFfiObject(
			pointer,
			func(pointer unsafe.Pointer, status *C.RustCallStatus) {
				C.uniffi_iroh_fn_free_irohnode(pointer, status)
			}),
	}
	runtime.SetFinalizer(result, (*IrohNode).Destroy)
	return result
}

func (c FfiConverterIrohNode) Read(reader io.Reader) *IrohNode {
	return c.Lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterIrohNode) Lower(value *IrohNode) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*IrohNode")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterIrohNode) Write(writer io.Writer, value *IrohNode) {
	writeUint64(writer, uint64(uintptr(c.Lower(value))))
}

type FfiDestroyerIrohNode struct{}

func (_ FfiDestroyerIrohNode) Destroy(value *IrohNode) {
	value.Destroy()
}

type PublicKey struct {
	ffiObject FfiObject
}

func (_self *PublicKey) ToBytes() []byte {
	_pointer := _self.ffiObject.incrementPointer("*PublicKey")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterBytesINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_publickey_to_bytes(
			_pointer, _uniffiStatus)
	}))

}

func (_self *PublicKey) ToString() string {
	_pointer := _self.ffiObject.incrementPointer("*PublicKey")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return C.uniffi_iroh_fn_method_publickey_to_string(
			_pointer, _uniffiStatus)
	}))

}

func (object *PublicKey) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterPublicKey struct{}

var FfiConverterPublicKeyINSTANCE = FfiConverterPublicKey{}

func (c FfiConverterPublicKey) Lift(pointer unsafe.Pointer) *PublicKey {
	result := &PublicKey{
		newFfiObject(
			pointer,
			func(pointer unsafe.Pointer, status *C.RustCallStatus) {
				C.uniffi_iroh_fn_free_publickey(pointer, status)
			}),
	}
	runtime.SetFinalizer(result, (*PublicKey).Destroy)
	return result
}

func (c FfiConverterPublicKey) Read(reader io.Reader) *PublicKey {
	return c.Lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterPublicKey) Lower(value *PublicKey) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*PublicKey")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterPublicKey) Write(writer io.Writer, value *PublicKey) {
	writeUint64(writer, uint64(uintptr(c.Lower(value))))
}

type FfiDestroyerPublicKey struct{}

func (_ FfiDestroyerPublicKey) Destroy(value *PublicKey) {
	value.Destroy()
}

type ConnectionInfo struct {
	Id         uint64
	PublicKey  *PublicKey
	DerpRegion *uint16
	Addrs      []SocketAddr
	Latencies  []*float64
	ConnType   ConnectionType
	Latency    *float64
}

func (r *ConnectionInfo) Destroy() {
	FfiDestroyerUint64{}.Destroy(r.Id)
	FfiDestroyerPublicKey{}.Destroy(r.PublicKey)
	FfiDestroyerOptionalUint16{}.Destroy(r.DerpRegion)
	FfiDestroyerSequenceTypeSocketAddr{}.Destroy(r.Addrs)
	FfiDestroyerSequenceOptionalFloat64{}.Destroy(r.Latencies)
	FfiDestroyerTypeConnectionType{}.Destroy(r.ConnType)
	FfiDestroyerOptionalFloat64{}.Destroy(r.Latency)
}

type FfiConverterTypeConnectionInfo struct{}

var FfiConverterTypeConnectionInfoINSTANCE = FfiConverterTypeConnectionInfo{}

func (c FfiConverterTypeConnectionInfo) Lift(rb RustBufferI) ConnectionInfo {
	return LiftFromRustBuffer[ConnectionInfo](c, rb)
}

func (c FfiConverterTypeConnectionInfo) Read(reader io.Reader) ConnectionInfo {
	return ConnectionInfo{
		FfiConverterUint64INSTANCE.Read(reader),
		FfiConverterPublicKeyINSTANCE.Read(reader),
		FfiConverterOptionalUint16INSTANCE.Read(reader),
		FfiConverterSequenceTypeSocketAddrINSTANCE.Read(reader),
		FfiConverterSequenceOptionalFloat64INSTANCE.Read(reader),
		FfiConverterTypeConnectionTypeINSTANCE.Read(reader),
		FfiConverterOptionalFloat64INSTANCE.Read(reader),
	}
}

func (c FfiConverterTypeConnectionInfo) Lower(value ConnectionInfo) RustBuffer {
	return LowerIntoRustBuffer[ConnectionInfo](c, value)
}

func (c FfiConverterTypeConnectionInfo) Write(writer io.Writer, value ConnectionInfo) {
	FfiConverterUint64INSTANCE.Write(writer, value.Id)
	FfiConverterPublicKeyINSTANCE.Write(writer, value.PublicKey)
	FfiConverterOptionalUint16INSTANCE.Write(writer, value.DerpRegion)
	FfiConverterSequenceTypeSocketAddrINSTANCE.Write(writer, value.Addrs)
	FfiConverterSequenceOptionalFloat64INSTANCE.Write(writer, value.Latencies)
	FfiConverterTypeConnectionTypeINSTANCE.Write(writer, value.ConnType)
	FfiConverterOptionalFloat64INSTANCE.Write(writer, value.Latency)
}

type FfiDestroyerTypeConnectionInfo struct{}

func (_ FfiDestroyerTypeConnectionInfo) Destroy(value ConnectionInfo) {
	value.Destroy()
}

type CounterStats struct {
	Value       uint64
	Description string
}

func (r *CounterStats) Destroy() {
	FfiDestroyerUint64{}.Destroy(r.Value)
	FfiDestroyerString{}.Destroy(r.Description)
}

type FfiConverterTypeCounterStats struct{}

var FfiConverterTypeCounterStatsINSTANCE = FfiConverterTypeCounterStats{}

func (c FfiConverterTypeCounterStats) Lift(rb RustBufferI) CounterStats {
	return LiftFromRustBuffer[CounterStats](c, rb)
}

func (c FfiConverterTypeCounterStats) Read(reader io.Reader) CounterStats {
	return CounterStats{
		FfiConverterUint64INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterTypeCounterStats) Lower(value CounterStats) RustBuffer {
	return LowerIntoRustBuffer[CounterStats](c, value)
}

func (c FfiConverterTypeCounterStats) Write(writer io.Writer, value CounterStats) {
	FfiConverterUint64INSTANCE.Write(writer, value.Value)
	FfiConverterStringINSTANCE.Write(writer, value.Description)
}

type FfiDestroyerTypeCounterStats struct{}

func (_ FfiDestroyerTypeCounterStats) Destroy(value CounterStats) {
	value.Destroy()
}

type LiveStatus struct {
	Active        bool
	Subscriptions uint64
}

func (r *LiveStatus) Destroy() {
	FfiDestroyerBool{}.Destroy(r.Active)
	FfiDestroyerUint64{}.Destroy(r.Subscriptions)
}

type FfiConverterTypeLiveStatus struct{}

var FfiConverterTypeLiveStatusINSTANCE = FfiConverterTypeLiveStatus{}

func (c FfiConverterTypeLiveStatus) Lift(rb RustBufferI) LiveStatus {
	return LiftFromRustBuffer[LiveStatus](c, rb)
}

func (c FfiConverterTypeLiveStatus) Read(reader io.Reader) LiveStatus {
	return LiveStatus{
		FfiConverterBoolINSTANCE.Read(reader),
		FfiConverterUint64INSTANCE.Read(reader),
	}
}

func (c FfiConverterTypeLiveStatus) Lower(value LiveStatus) RustBuffer {
	return LowerIntoRustBuffer[LiveStatus](c, value)
}

func (c FfiConverterTypeLiveStatus) Write(writer io.Writer, value LiveStatus) {
	FfiConverterBoolINSTANCE.Write(writer, value.Active)
	FfiConverterUint64INSTANCE.Write(writer, value.Subscriptions)
}

type FfiDestroyerTypeLiveStatus struct{}

func (_ FfiDestroyerTypeLiveStatus) Destroy(value LiveStatus) {
	value.Destroy()
}

type ConnectionType interface {
	Destroy()
}
type ConnectionTypeDirect struct {
	Addr SocketAddr
}

func (e ConnectionTypeDirect) Destroy() {
	FfiDestroyerTypeSocketAddr{}.Destroy(e.Addr)
}

type ConnectionTypeRelay struct {
	Port uint16
}

func (e ConnectionTypeRelay) Destroy() {
	FfiDestroyerUint16{}.Destroy(e.Port)
}

type ConnectionTypeNone struct {
}

func (e ConnectionTypeNone) Destroy() {
}

type FfiConverterTypeConnectionType struct{}

var FfiConverterTypeConnectionTypeINSTANCE = FfiConverterTypeConnectionType{}

func (c FfiConverterTypeConnectionType) Lift(rb RustBufferI) ConnectionType {
	return LiftFromRustBuffer[ConnectionType](c, rb)
}

func (c FfiConverterTypeConnectionType) Lower(value ConnectionType) RustBuffer {
	return LowerIntoRustBuffer[ConnectionType](c, value)
}
func (FfiConverterTypeConnectionType) Read(reader io.Reader) ConnectionType {
	id := readInt32(reader)
	switch id {
	case 1:
		return ConnectionTypeDirect{
			FfiConverterTypeSocketAddrINSTANCE.Read(reader),
		}
	case 2:
		return ConnectionTypeRelay{
			FfiConverterUint16INSTANCE.Read(reader),
		}
	case 3:
		return ConnectionTypeNone{}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterTypeConnectionType.Read()", id))
	}
}

func (FfiConverterTypeConnectionType) Write(writer io.Writer, value ConnectionType) {
	switch variant_value := value.(type) {
	case ConnectionTypeDirect:
		writeInt32(writer, 1)
		FfiConverterTypeSocketAddrINSTANCE.Write(writer, variant_value.Addr)
	case ConnectionTypeRelay:
		writeInt32(writer, 2)
		FfiConverterUint16INSTANCE.Write(writer, variant_value.Port)
	case ConnectionTypeNone:
		writeInt32(writer, 3)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterTypeConnectionType.Write", value))
	}
}

type FfiDestroyerTypeConnectionType struct{}

func (_ FfiDestroyerTypeConnectionType) Destroy(value ConnectionType) {
	value.Destroy()
}

type IrohError struct {
	err error
}

func (err IrohError) Error() string {
	return fmt.Sprintf("IrohError: %s", err.err.Error())
}

func (err IrohError) Unwrap() error {
	return err.err
}

// Err* are used for checking error type with `errors.Is`
var ErrIrohErrorRuntime = fmt.Errorf("IrohErrorRuntime")
var ErrIrohErrorNodeCreate = fmt.Errorf("IrohErrorNodeCreate")
var ErrIrohErrorDoc = fmt.Errorf("IrohErrorDoc")
var ErrIrohErrorAuthor = fmt.Errorf("IrohErrorAuthor")
var ErrIrohErrorDocTicket = fmt.Errorf("IrohErrorDocTicket")
var ErrIrohErrorUniffi = fmt.Errorf("IrohErrorUniffi")
var ErrIrohErrorConnection = fmt.Errorf("IrohErrorConnection")

// Variant structs
type IrohErrorRuntime struct {
	Description string
}

func NewIrohErrorRuntime(
	description string,
) *IrohError {
	return &IrohError{
		err: &IrohErrorRuntime{
			Description: description,
		},
	}
}

func (err IrohErrorRuntime) Error() string {
	return fmt.Sprint("Runtime",
		": ",

		"Description=",
		err.Description,
	)
}

func (self IrohErrorRuntime) Is(target error) bool {
	return target == ErrIrohErrorRuntime
}

type IrohErrorNodeCreate struct {
	Description string
}

func NewIrohErrorNodeCreate(
	description string,
) *IrohError {
	return &IrohError{
		err: &IrohErrorNodeCreate{
			Description: description,
		},
	}
}

func (err IrohErrorNodeCreate) Error() string {
	return fmt.Sprint("NodeCreate",
		": ",

		"Description=",
		err.Description,
	)
}

func (self IrohErrorNodeCreate) Is(target error) bool {
	return target == ErrIrohErrorNodeCreate
}

type IrohErrorDoc struct {
	Description string
}

func NewIrohErrorDoc(
	description string,
) *IrohError {
	return &IrohError{
		err: &IrohErrorDoc{
			Description: description,
		},
	}
}

func (err IrohErrorDoc) Error() string {
	return fmt.Sprint("Doc",
		": ",

		"Description=",
		err.Description,
	)
}

func (self IrohErrorDoc) Is(target error) bool {
	return target == ErrIrohErrorDoc
}

type IrohErrorAuthor struct {
	Description string
}

func NewIrohErrorAuthor(
	description string,
) *IrohError {
	return &IrohError{
		err: &IrohErrorAuthor{
			Description: description,
		},
	}
}

func (err IrohErrorAuthor) Error() string {
	return fmt.Sprint("Author",
		": ",

		"Description=",
		err.Description,
	)
}

func (self IrohErrorAuthor) Is(target error) bool {
	return target == ErrIrohErrorAuthor
}

type IrohErrorDocTicket struct {
	Description string
}

func NewIrohErrorDocTicket(
	description string,
) *IrohError {
	return &IrohError{
		err: &IrohErrorDocTicket{
			Description: description,
		},
	}
}

func (err IrohErrorDocTicket) Error() string {
	return fmt.Sprint("DocTicket",
		": ",

		"Description=",
		err.Description,
	)
}

func (self IrohErrorDocTicket) Is(target error) bool {
	return target == ErrIrohErrorDocTicket
}

type IrohErrorUniffi struct {
	Description string
}

func NewIrohErrorUniffi(
	description string,
) *IrohError {
	return &IrohError{
		err: &IrohErrorUniffi{
			Description: description,
		},
	}
}

func (err IrohErrorUniffi) Error() string {
	return fmt.Sprint("Uniffi",
		": ",

		"Description=",
		err.Description,
	)
}

func (self IrohErrorUniffi) Is(target error) bool {
	return target == ErrIrohErrorUniffi
}

type IrohErrorConnection struct {
	Description string
}

func NewIrohErrorConnection(
	description string,
) *IrohError {
	return &IrohError{
		err: &IrohErrorConnection{
			Description: description,
		},
	}
}

func (err IrohErrorConnection) Error() string {
	return fmt.Sprint("Connection",
		": ",

		"Description=",
		err.Description,
	)
}

func (self IrohErrorConnection) Is(target error) bool {
	return target == ErrIrohErrorConnection
}

type FfiConverterTypeIrohError struct{}

var FfiConverterTypeIrohErrorINSTANCE = FfiConverterTypeIrohError{}

func (c FfiConverterTypeIrohError) Lift(eb RustBufferI) error {
	return LiftFromRustBuffer[error](c, eb)
}

func (c FfiConverterTypeIrohError) Lower(value *IrohError) RustBuffer {
	return LowerIntoRustBuffer[*IrohError](c, value)
}

func (c FfiConverterTypeIrohError) Read(reader io.Reader) error {
	errorID := readUint32(reader)

	switch errorID {
	case 1:
		return &IrohError{&IrohErrorRuntime{
			Description: FfiConverterStringINSTANCE.Read(reader),
		}}
	case 2:
		return &IrohError{&IrohErrorNodeCreate{
			Description: FfiConverterStringINSTANCE.Read(reader),
		}}
	case 3:
		return &IrohError{&IrohErrorDoc{
			Description: FfiConverterStringINSTANCE.Read(reader),
		}}
	case 4:
		return &IrohError{&IrohErrorAuthor{
			Description: FfiConverterStringINSTANCE.Read(reader),
		}}
	case 5:
		return &IrohError{&IrohErrorDocTicket{
			Description: FfiConverterStringINSTANCE.Read(reader),
		}}
	case 6:
		return &IrohError{&IrohErrorUniffi{
			Description: FfiConverterStringINSTANCE.Read(reader),
		}}
	case 7:
		return &IrohError{&IrohErrorConnection{
			Description: FfiConverterStringINSTANCE.Read(reader),
		}}
	default:
		panic(fmt.Sprintf("Unknown error code %d in FfiConverterTypeIrohError.Read()", errorID))
	}
}

func (c FfiConverterTypeIrohError) Write(writer io.Writer, value *IrohError) {
	switch variantValue := value.err.(type) {
	case *IrohErrorRuntime:
		writeInt32(writer, 1)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Description)
	case *IrohErrorNodeCreate:
		writeInt32(writer, 2)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Description)
	case *IrohErrorDoc:
		writeInt32(writer, 3)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Description)
	case *IrohErrorAuthor:
		writeInt32(writer, 4)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Description)
	case *IrohErrorDocTicket:
		writeInt32(writer, 5)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Description)
	case *IrohErrorUniffi:
		writeInt32(writer, 6)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Description)
	case *IrohErrorConnection:
		writeInt32(writer, 7)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Description)
	default:
		_ = variantValue
		panic(fmt.Sprintf("invalid error value `%v` in FfiConverterTypeIrohError.Write", value))
	}
}

type LiveEvent uint

const (
	LiveEventInsertLocal  LiveEvent = 1
	LiveEventInsertRemote LiveEvent = 2
	LiveEventContentReady LiveEvent = 3
)

type FfiConverterTypeLiveEvent struct{}

var FfiConverterTypeLiveEventINSTANCE = FfiConverterTypeLiveEvent{}

func (c FfiConverterTypeLiveEvent) Lift(rb RustBufferI) LiveEvent {
	return LiftFromRustBuffer[LiveEvent](c, rb)
}

func (c FfiConverterTypeLiveEvent) Lower(value LiveEvent) RustBuffer {
	return LowerIntoRustBuffer[LiveEvent](c, value)
}
func (FfiConverterTypeLiveEvent) Read(reader io.Reader) LiveEvent {
	id := readInt32(reader)
	return LiveEvent(id)
}

func (FfiConverterTypeLiveEvent) Write(writer io.Writer, value LiveEvent) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerTypeLiveEvent struct{}

func (_ FfiDestroyerTypeLiveEvent) Destroy(value LiveEvent) {
}

type LogLevel uint

const (
	LogLevelTrace LogLevel = 1
	LogLevelDebug LogLevel = 2
	LogLevelInfo  LogLevel = 3
	LogLevelWarn  LogLevel = 4
	LogLevelError LogLevel = 5
	LogLevelOff   LogLevel = 6
)

type FfiConverterTypeLogLevel struct{}

var FfiConverterTypeLogLevelINSTANCE = FfiConverterTypeLogLevel{}

func (c FfiConverterTypeLogLevel) Lift(rb RustBufferI) LogLevel {
	return LiftFromRustBuffer[LogLevel](c, rb)
}

func (c FfiConverterTypeLogLevel) Lower(value LogLevel) RustBuffer {
	return LowerIntoRustBuffer[LogLevel](c, value)
}
func (FfiConverterTypeLogLevel) Read(reader io.Reader) LogLevel {
	id := readInt32(reader)
	return LogLevel(id)
}

func (FfiConverterTypeLogLevel) Write(writer io.Writer, value LogLevel) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerTypeLogLevel struct{}

func (_ FfiDestroyerTypeLogLevel) Destroy(value LogLevel) {
}

type SocketAddr interface {
	Destroy()
}
type SocketAddrV4 struct {
	A uint8
	B uint8
	C uint8
	D uint8
}

func (e SocketAddrV4) Destroy() {
	FfiDestroyerUint8{}.Destroy(e.A)
	FfiDestroyerUint8{}.Destroy(e.B)
	FfiDestroyerUint8{}.Destroy(e.C)
	FfiDestroyerUint8{}.Destroy(e.D)
}

type SocketAddrV6 struct {
	Addr []byte
}

func (e SocketAddrV6) Destroy() {
	FfiDestroyerBytes{}.Destroy(e.Addr)
}

type FfiConverterTypeSocketAddr struct{}

var FfiConverterTypeSocketAddrINSTANCE = FfiConverterTypeSocketAddr{}

func (c FfiConverterTypeSocketAddr) Lift(rb RustBufferI) SocketAddr {
	return LiftFromRustBuffer[SocketAddr](c, rb)
}

func (c FfiConverterTypeSocketAddr) Lower(value SocketAddr) RustBuffer {
	return LowerIntoRustBuffer[SocketAddr](c, value)
}
func (FfiConverterTypeSocketAddr) Read(reader io.Reader) SocketAddr {
	id := readInt32(reader)
	switch id {
	case 1:
		return SocketAddrV4{
			FfiConverterUint8INSTANCE.Read(reader),
			FfiConverterUint8INSTANCE.Read(reader),
			FfiConverterUint8INSTANCE.Read(reader),
			FfiConverterUint8INSTANCE.Read(reader),
		}
	case 2:
		return SocketAddrV6{
			FfiConverterBytesINSTANCE.Read(reader),
		}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterTypeSocketAddr.Read()", id))
	}
}

func (FfiConverterTypeSocketAddr) Write(writer io.Writer, value SocketAddr) {
	switch variant_value := value.(type) {
	case SocketAddrV4:
		writeInt32(writer, 1)
		FfiConverterUint8INSTANCE.Write(writer, variant_value.A)
		FfiConverterUint8INSTANCE.Write(writer, variant_value.B)
		FfiConverterUint8INSTANCE.Write(writer, variant_value.C)
		FfiConverterUint8INSTANCE.Write(writer, variant_value.D)
	case SocketAddrV6:
		writeInt32(writer, 2)
		FfiConverterBytesINSTANCE.Write(writer, variant_value.Addr)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterTypeSocketAddr.Write", value))
	}
}

type FfiDestroyerTypeSocketAddr struct{}

func (_ FfiDestroyerTypeSocketAddr) Destroy(value SocketAddr) {
	value.Destroy()
}

type uniffiCallbackResult C.int32_t

const (
	idxCallbackFree                                          = 0
	uniffiCallbackResultSuccess         uniffiCallbackResult = 0
	uniffiCallbackResultError           uniffiCallbackResult = 1
	uniffiCallbackUnexpectedResultError uniffiCallbackResult = 2
)

type concurrentHandleMap[T any] struct {
	leftMap       map[uint64]*T
	rightMap      map[*T]uint64
	currentHandle uint64
	lock          sync.Mutex
}

func newConcurrentHandleMap[T any]() *concurrentHandleMap[T] {
	return &concurrentHandleMap[T]{
		leftMap:  map[uint64]*T{},
		rightMap: map[*T]uint64{},
	}
}

func (cm *concurrentHandleMap[T]) insert(obj *T) uint64 {
	cm.lock.Lock()
	defer cm.lock.Unlock()
	if existingHandle, ok := cm.rightMap[obj]; ok {
		return existingHandle
	}
	cm.currentHandle = cm.currentHandle + 1
	cm.leftMap[cm.currentHandle] = obj
	cm.rightMap[obj] = cm.currentHandle
	return cm.currentHandle
}

func (cm *concurrentHandleMap[T]) remove(handle uint64) bool {
	cm.lock.Lock()
	defer cm.lock.Unlock()
	if val, ok := cm.leftMap[handle]; ok {
		delete(cm.leftMap, handle)
		delete(cm.rightMap, val)
	}
	return false
}

func (cm *concurrentHandleMap[T]) tryGet(handle uint64) (*T, bool) {
	val, ok := cm.leftMap[handle]
	return val, ok
}

type FfiConverterCallbackInterface[CallbackInterface any] struct {
	handleMap *concurrentHandleMap[CallbackInterface]
}

func (c *FfiConverterCallbackInterface[CallbackInterface]) drop(handle uint64) RustBuffer {
	c.handleMap.remove(handle)
	return RustBuffer{}
}

func (c *FfiConverterCallbackInterface[CallbackInterface]) Lift(handle uint64) CallbackInterface {
	val, ok := c.handleMap.tryGet(handle)
	if !ok {
		panic(fmt.Errorf("no callback in handle map: %d", handle))
	}
	return *val
}

func (c *FfiConverterCallbackInterface[CallbackInterface]) Read(reader io.Reader) CallbackInterface {
	return c.Lift(readUint64(reader))
}

func (c *FfiConverterCallbackInterface[CallbackInterface]) Lower(value CallbackInterface) C.uint64_t {
	return C.uint64_t(c.handleMap.insert(&value))
}

func (c *FfiConverterCallbackInterface[CallbackInterface]) Write(writer io.Writer, value CallbackInterface) {
	writeUint64(writer, uint64(c.Lower(value)))
}

// Declaration and FfiConverters for SubscribeCallback Callback Interface
type SubscribeCallback interface {
	Event(
		event LiveEvent) *IrohError
}

// foreignCallbackCallbackInterfaceSubscribeCallback cannot be callable be a compiled function at a same time
type foreignCallbackCallbackInterfaceSubscribeCallback struct{}

//export iroh_cgo_SubscribeCallback
func iroh_cgo_SubscribeCallback(handle C.uint64_t, method C.int32_t, argsPtr *C.uint8_t, argsLen C.int32_t, outBuf *C.RustBuffer) C.int32_t {
	cb := FfiConverterCallbackInterfaceSubscribeCallbackINSTANCE.Lift(uint64(handle))
	switch method {
	case 0:
		// 0 means Rust is done with the callback, and the callback
		// can be dropped by the foreign language.
		*outBuf = FfiConverterCallbackInterfaceSubscribeCallbackINSTANCE.drop(uint64(handle))
		// See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
		return C.int32_t(idxCallbackFree)

	case 1:
		var result uniffiCallbackResult
		args := unsafe.Slice((*byte)(argsPtr), argsLen)
		result = foreignCallbackCallbackInterfaceSubscribeCallback{}.InvokeEvent(cb, args, outBuf)
		return C.int32_t(result)

	default:
		// This should never happen, because an out of bounds method index won't
		// ever be used. Once we can catch errors, we should return an InternalException.
		// https://github.com/mozilla/uniffi-rs/issues/351
		return C.int32_t(uniffiCallbackUnexpectedResultError)
	}
}

func (foreignCallbackCallbackInterfaceSubscribeCallback) InvokeEvent(callback SubscribeCallback, args []byte, outBuf *C.RustBuffer) uniffiCallbackResult {
	reader := bytes.NewReader(args)
	err := callback.Event(FfiConverterTypeLiveEventINSTANCE.Read(reader))

	if err != nil {
		// The only way to bypass an unexpected error is to bypass pointer to an empty
		// instance of the error
		if err.err == nil {
			return uniffiCallbackUnexpectedResultError
		}
		*outBuf = LowerIntoRustBuffer[*IrohError](FfiConverterTypeIrohErrorINSTANCE, err)
		return uniffiCallbackResultError
	}
	return uniffiCallbackResultSuccess
}

type FfiConverterCallbackInterfaceSubscribeCallback struct {
	FfiConverterCallbackInterface[SubscribeCallback]
}

var FfiConverterCallbackInterfaceSubscribeCallbackINSTANCE = &FfiConverterCallbackInterfaceSubscribeCallback{
	FfiConverterCallbackInterface: FfiConverterCallbackInterface[SubscribeCallback]{
		handleMap: newConcurrentHandleMap[SubscribeCallback](),
	},
}

// This is a static function because only 1 instance is supported for registering
func (c *FfiConverterCallbackInterfaceSubscribeCallback) register() {
	rustCall(func(status *C.RustCallStatus) int32 {
		C.uniffi_iroh_fn_init_callback_subscribecallback(C.ForeignCallback(C.iroh_cgo_SubscribeCallback), status)
		return 0
	})
}

type FfiDestroyerCallbackInterfaceSubscribeCallback struct{}

func (FfiDestroyerCallbackInterfaceSubscribeCallback) Destroy(value SubscribeCallback) {
}

type FfiConverterOptionalUint16 struct{}

var FfiConverterOptionalUint16INSTANCE = FfiConverterOptionalUint16{}

func (c FfiConverterOptionalUint16) Lift(rb RustBufferI) *uint16 {
	return LiftFromRustBuffer[*uint16](c, rb)
}

func (_ FfiConverterOptionalUint16) Read(reader io.Reader) *uint16 {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterUint16INSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalUint16) Lower(value *uint16) RustBuffer {
	return LowerIntoRustBuffer[*uint16](c, value)
}

func (_ FfiConverterOptionalUint16) Write(writer io.Writer, value *uint16) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterUint16INSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalUint16 struct{}

func (_ FfiDestroyerOptionalUint16) Destroy(value *uint16) {
	if value != nil {
		FfiDestroyerUint16{}.Destroy(*value)
	}
}

type FfiConverterOptionalFloat64 struct{}

var FfiConverterOptionalFloat64INSTANCE = FfiConverterOptionalFloat64{}

func (c FfiConverterOptionalFloat64) Lift(rb RustBufferI) *float64 {
	return LiftFromRustBuffer[*float64](c, rb)
}

func (_ FfiConverterOptionalFloat64) Read(reader io.Reader) *float64 {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterFloat64INSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalFloat64) Lower(value *float64) RustBuffer {
	return LowerIntoRustBuffer[*float64](c, value)
}

func (_ FfiConverterOptionalFloat64) Write(writer io.Writer, value *float64) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterFloat64INSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalFloat64 struct{}

func (_ FfiDestroyerOptionalFloat64) Destroy(value *float64) {
	if value != nil {
		FfiDestroyerFloat64{}.Destroy(*value)
	}
}

type FfiConverterOptionalTypeConnectionInfo struct{}

var FfiConverterOptionalTypeConnectionInfoINSTANCE = FfiConverterOptionalTypeConnectionInfo{}

func (c FfiConverterOptionalTypeConnectionInfo) Lift(rb RustBufferI) *ConnectionInfo {
	return LiftFromRustBuffer[*ConnectionInfo](c, rb)
}

func (_ FfiConverterOptionalTypeConnectionInfo) Read(reader io.Reader) *ConnectionInfo {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterTypeConnectionInfoINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalTypeConnectionInfo) Lower(value *ConnectionInfo) RustBuffer {
	return LowerIntoRustBuffer[*ConnectionInfo](c, value)
}

func (_ FfiConverterOptionalTypeConnectionInfo) Write(writer io.Writer, value *ConnectionInfo) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterTypeConnectionInfoINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalTypeConnectionInfo struct{}

func (_ FfiDestroyerOptionalTypeConnectionInfo) Destroy(value *ConnectionInfo) {
	if value != nil {
		FfiDestroyerTypeConnectionInfo{}.Destroy(*value)
	}
}

type FfiConverterSequenceAuthorId struct{}

var FfiConverterSequenceAuthorIdINSTANCE = FfiConverterSequenceAuthorId{}

func (c FfiConverterSequenceAuthorId) Lift(rb RustBufferI) []*AuthorId {
	return LiftFromRustBuffer[[]*AuthorId](c, rb)
}

func (c FfiConverterSequenceAuthorId) Read(reader io.Reader) []*AuthorId {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]*AuthorId, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterAuthorIdINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceAuthorId) Lower(value []*AuthorId) RustBuffer {
	return LowerIntoRustBuffer[[]*AuthorId](c, value)
}

func (c FfiConverterSequenceAuthorId) Write(writer io.Writer, value []*AuthorId) {
	if len(value) > math.MaxInt32 {
		panic("[]*AuthorId is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterAuthorIdINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceAuthorId struct{}

func (FfiDestroyerSequenceAuthorId) Destroy(sequence []*AuthorId) {
	for _, value := range sequence {
		FfiDestroyerAuthorId{}.Destroy(value)
	}
}

type FfiConverterSequenceEntry struct{}

var FfiConverterSequenceEntryINSTANCE = FfiConverterSequenceEntry{}

func (c FfiConverterSequenceEntry) Lift(rb RustBufferI) []*Entry {
	return LiftFromRustBuffer[[]*Entry](c, rb)
}

func (c FfiConverterSequenceEntry) Read(reader io.Reader) []*Entry {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]*Entry, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterEntryINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceEntry) Lower(value []*Entry) RustBuffer {
	return LowerIntoRustBuffer[[]*Entry](c, value)
}

func (c FfiConverterSequenceEntry) Write(writer io.Writer, value []*Entry) {
	if len(value) > math.MaxInt32 {
		panic("[]*Entry is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterEntryINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceEntry struct{}

func (FfiDestroyerSequenceEntry) Destroy(sequence []*Entry) {
	for _, value := range sequence {
		FfiDestroyerEntry{}.Destroy(value)
	}
}

type FfiConverterSequenceTypeConnectionInfo struct{}

var FfiConverterSequenceTypeConnectionInfoINSTANCE = FfiConverterSequenceTypeConnectionInfo{}

func (c FfiConverterSequenceTypeConnectionInfo) Lift(rb RustBufferI) []ConnectionInfo {
	return LiftFromRustBuffer[[]ConnectionInfo](c, rb)
}

func (c FfiConverterSequenceTypeConnectionInfo) Read(reader io.Reader) []ConnectionInfo {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]ConnectionInfo, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterTypeConnectionInfoINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceTypeConnectionInfo) Lower(value []ConnectionInfo) RustBuffer {
	return LowerIntoRustBuffer[[]ConnectionInfo](c, value)
}

func (c FfiConverterSequenceTypeConnectionInfo) Write(writer io.Writer, value []ConnectionInfo) {
	if len(value) > math.MaxInt32 {
		panic("[]ConnectionInfo is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterTypeConnectionInfoINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceTypeConnectionInfo struct{}

func (FfiDestroyerSequenceTypeConnectionInfo) Destroy(sequence []ConnectionInfo) {
	for _, value := range sequence {
		FfiDestroyerTypeConnectionInfo{}.Destroy(value)
	}
}

type FfiConverterSequenceTypeSocketAddr struct{}

var FfiConverterSequenceTypeSocketAddrINSTANCE = FfiConverterSequenceTypeSocketAddr{}

func (c FfiConverterSequenceTypeSocketAddr) Lift(rb RustBufferI) []SocketAddr {
	return LiftFromRustBuffer[[]SocketAddr](c, rb)
}

func (c FfiConverterSequenceTypeSocketAddr) Read(reader io.Reader) []SocketAddr {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]SocketAddr, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterTypeSocketAddrINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceTypeSocketAddr) Lower(value []SocketAddr) RustBuffer {
	return LowerIntoRustBuffer[[]SocketAddr](c, value)
}

func (c FfiConverterSequenceTypeSocketAddr) Write(writer io.Writer, value []SocketAddr) {
	if len(value) > math.MaxInt32 {
		panic("[]SocketAddr is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterTypeSocketAddrINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceTypeSocketAddr struct{}

func (FfiDestroyerSequenceTypeSocketAddr) Destroy(sequence []SocketAddr) {
	for _, value := range sequence {
		FfiDestroyerTypeSocketAddr{}.Destroy(value)
	}
}

type FfiConverterSequenceOptionalFloat64 struct{}

var FfiConverterSequenceOptionalFloat64INSTANCE = FfiConverterSequenceOptionalFloat64{}

func (c FfiConverterSequenceOptionalFloat64) Lift(rb RustBufferI) []*float64 {
	return LiftFromRustBuffer[[]*float64](c, rb)
}

func (c FfiConverterSequenceOptionalFloat64) Read(reader io.Reader) []*float64 {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]*float64, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterOptionalFloat64INSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceOptionalFloat64) Lower(value []*float64) RustBuffer {
	return LowerIntoRustBuffer[[]*float64](c, value)
}

func (c FfiConverterSequenceOptionalFloat64) Write(writer io.Writer, value []*float64) {
	if len(value) > math.MaxInt32 {
		panic("[]*float64 is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterOptionalFloat64INSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceOptionalFloat64 struct{}

func (FfiDestroyerSequenceOptionalFloat64) Destroy(sequence []*float64) {
	for _, value := range sequence {
		FfiDestroyerOptionalFloat64{}.Destroy(value)
	}
}

type FfiConverterMapStringTypeCounterStats struct{}

var FfiConverterMapStringTypeCounterStatsINSTANCE = FfiConverterMapStringTypeCounterStats{}

func (c FfiConverterMapStringTypeCounterStats) Lift(rb RustBufferI) map[string]CounterStats {
	return LiftFromRustBuffer[map[string]CounterStats](c, rb)
}

func (_ FfiConverterMapStringTypeCounterStats) Read(reader io.Reader) map[string]CounterStats {
	result := make(map[string]CounterStats)
	length := readInt32(reader)
	for i := int32(0); i < length; i++ {
		key := FfiConverterStringINSTANCE.Read(reader)
		value := FfiConverterTypeCounterStatsINSTANCE.Read(reader)
		result[key] = value
	}
	return result
}

func (c FfiConverterMapStringTypeCounterStats) Lower(value map[string]CounterStats) RustBuffer {
	return LowerIntoRustBuffer[map[string]CounterStats](c, value)
}

func (_ FfiConverterMapStringTypeCounterStats) Write(writer io.Writer, mapValue map[string]CounterStats) {
	if len(mapValue) > math.MaxInt32 {
		panic("map[string]CounterStats is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(mapValue)))
	for key, value := range mapValue {
		FfiConverterStringINSTANCE.Write(writer, key)
		FfiConverterTypeCounterStatsINSTANCE.Write(writer, value)
	}
}

type FfiDestroyerMapStringTypeCounterStats struct{}

func (_ FfiDestroyerMapStringTypeCounterStats) Destroy(mapValue map[string]CounterStats) {
	for key, value := range mapValue {
		FfiDestroyerString{}.Destroy(key)
		FfiDestroyerTypeCounterStats{}.Destroy(value)
	}
}

func SetLogLevel(
	level LogLevel) {

	rustCall(func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_iroh_fn_func_set_log_level(FfiConverterTypeLogLevelINSTANCE.Lower(level), _uniffiStatus)
		return false
	})

}
