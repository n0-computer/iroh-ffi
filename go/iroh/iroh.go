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

type rustBuffer struct {
	self C.RustBuffer
}

func fromCRustBuffer(crbuf C.RustBuffer) rustBuffer {
	capacity := int(crbuf.capacity)
	length := int(crbuf.len)
	data := unsafe.Pointer(crbuf.data)

	if data == nil && (capacity > 0 || length > 0) {
		panic(fmt.Sprintf("null in valid C.RustBuffer, capacity non null on null data: %d, %d, %s", capacity, length, data))
	}
	return rustBuffer{
		self: crbuf,
	}
}

// asByteBuffer reads the full rust buffer and then converts read bytes to a new reader which makes
// it quite inefficient
// TODO: Return an implementation which reads only when needed
func (rb rustBuffer) asReader() *bytes.Reader {
	b := C.GoBytes(unsafe.Pointer(rb.self.data), C.int(rb.self.len))
	return bytes.NewReader(b)
}

func (rb rustBuffer) asCRustBuffer() C.RustBuffer {
	return rb.self
}

func stringToCRustBuffer(str string) C.RustBuffer {
	return goBytesToCRustBuffer([]byte(str))
}

func (rb rustBuffer) free() {
	rustCall(func(status *C.RustCallStatus) bool {
		C.ffi_iroh_rustbuffer_free(rb.self, status)
		return false
	})
}

func goBytesToCRustBuffer(b []byte) C.RustBuffer {
	if len(b) == 0 {
		return C.RustBuffer{}
	}
	// We can pass the pointer along here, as it is pinned
	// for the duration of this call
	foreign := C.ForeignBytes{
		len:  C.int(len(b)),
		data: (*C.uchar)(unsafe.Pointer(&b[0])),
	}

	return rustCall(func(status *C.RustCallStatus) C.RustBuffer {
		return C.ffi_iroh_rustbuffer_from_bytes(foreign, status)
	})
}

func cRustBufferToGoBytes(b C.RustBuffer) []byte {
	return C.GoBytes(unsafe.Pointer(b.data), C.int(b.len))
}

type bufLifter[GoType any] interface {
	lift(value C.RustBuffer) GoType
}

type bufLowerer[GoType any] interface {
	lower(value GoType) C.RustBuffer
}

type ffiConverter[GoType any, FfiType any] interface {
	lift(value FfiType) GoType
	lower(value GoType) FfiType
}

type bufReader[GoType any] interface {
	read(reader io.Reader) GoType
}

type bufWriter[GoType any] interface {
	write(writer io.Writer, value GoType)
}

type ffiRustBufConverter[GoType any, FfiType any] interface {
	ffiConverter[GoType, FfiType]
	bufReader[GoType]
}

func lowerIntoRustBuffer[GoType any](bufWriter bufWriter[GoType], value GoType) C.RustBuffer {
	// This might be not the most efficient way but it does not require knowing allocation size
	// beforehand
	var buffer bytes.Buffer
	bufWriter.write(&buffer, value)

	bytes, err := io.ReadAll(&buffer)
	if err != nil {
		panic(fmt.Errorf("reading written data: %w", err))
	}
	return goBytesToCRustBuffer(bytes)
}

func liftFromRustBuffer[GoType any](bufReader bufReader[GoType], rbuf rustBuffer) GoType {
	defer rbuf.free()
	reader := rbuf.asReader()
	item := bufReader.read(reader)
	if reader.Len() > 0 {
		// TODO: Remove this
		leftover, _ := io.ReadAll(reader)
		panic(fmt.Errorf("Junk remaining in buffer after lifting: %s", string(leftover)))
	}
	return item
}

func rustCallWithError[U any](converter bufLifter[error], callback func(*C.RustCallStatus) U) (U, error) {
	var status C.RustCallStatus
	returnValue := callback(&status)
	switch status.code {
	case 0:
		return returnValue, nil
	case 1:
		return returnValue, converter.lift(status.errorBuf)
	case 2:
		// when the rust code sees a panic, it tries to construct a rustbuffer
		// with the message.  but if that code panics, then it just sends back
		// an empty buffer.
		if status.errorBuf.len > 0 {
			panic(fmt.Errorf("%s", FfiConverterstringINSTANCE.lift(status.errorBuf)))
		} else {
			panic(fmt.Errorf("Rust panicked while handling Rust panic"))
		}
	default:
		return returnValue, fmt.Errorf("unknown status code: %d", status.code)
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

	(&FfiConverterTypeSubscribeCallback{}).register()
}

type FfiConverteruint8 struct{}

var FfiConverteruint8INSTANCE = FfiConverteruint8{}

func (FfiConverteruint8) lower(value uint8) C.uint8_t {
	return C.uint8_t(value)
}

func (FfiConverteruint8) write(writer io.Writer, value uint8) {
	writeUint8(writer, value)
}

func (FfiConverteruint8) lift(value C.uint8_t) uint8 {
	return uint8(value)
}

func (FfiConverteruint8) read(reader io.Reader) uint8 {
	return readUint8(reader)
}

type FfiDestroyeruint8 struct{}

func (FfiDestroyeruint8) destroy(_ uint8) {}

type FfiConverteruint16 struct{}

var FfiConverteruint16INSTANCE = FfiConverteruint16{}

func (FfiConverteruint16) lower(value uint16) C.uint16_t {
	return C.uint16_t(value)
}

func (FfiConverteruint16) write(writer io.Writer, value uint16) {
	writeUint16(writer, value)
}

func (FfiConverteruint16) lift(value C.uint16_t) uint16 {
	return uint16(value)
}

func (FfiConverteruint16) read(reader io.Reader) uint16 {
	return readUint16(reader)
}

type FfiDestroyeruint16 struct{}

func (FfiDestroyeruint16) destroy(_ uint16) {}

type FfiConverteruint64 struct{}

var FfiConverteruint64INSTANCE = FfiConverteruint64{}

func (FfiConverteruint64) lower(value uint64) C.uint64_t {
	return C.uint64_t(value)
}

func (FfiConverteruint64) write(writer io.Writer, value uint64) {
	writeUint64(writer, value)
}

func (FfiConverteruint64) lift(value C.uint64_t) uint64 {
	return uint64(value)
}

func (FfiConverteruint64) read(reader io.Reader) uint64 {
	return readUint64(reader)
}

type FfiDestroyeruint64 struct{}

func (FfiDestroyeruint64) destroy(_ uint64) {}

type FfiConverterfloat64 struct{}

var FfiConverterfloat64INSTANCE = FfiConverterfloat64{}

func (FfiConverterfloat64) lower(value float64) C.double {
	return C.double(value)
}

func (FfiConverterfloat64) write(writer io.Writer, value float64) {
	writeFloat64(writer, value)
}

func (FfiConverterfloat64) lift(value C.double) float64 {
	return float64(value)
}

func (FfiConverterfloat64) read(reader io.Reader) float64 {
	return readFloat64(reader)
}

type FfiDestroyerfloat64 struct{}

func (FfiDestroyerfloat64) destroy(_ float64) {}

type FfiConverterbool struct{}

var FfiConverterboolINSTANCE = FfiConverterbool{}

func (FfiConverterbool) lower(value bool) C.int8_t {
	if value {
		return C.int8_t(1)
	}
	return C.int8_t(0)
}

func (FfiConverterbool) write(writer io.Writer, value bool) {
	if value {
		writeInt8(writer, 1)
	} else {
		writeInt8(writer, 0)
	}
}

func (FfiConverterbool) lift(value C.int8_t) bool {
	return value != 0
}

func (FfiConverterbool) read(reader io.Reader) bool {
	return readInt8(reader) != 0
}

type FfiDestroyerbool struct{}

func (FfiDestroyerbool) destroy(_ bool) {}

type FfiConverterstring struct{}

var FfiConverterstringINSTANCE = FfiConverterstring{}

func (FfiConverterstring) lift(cRustBuf C.RustBuffer) string {
	reader := fromCRustBuffer(cRustBuf).asReader()
	b, err := io.ReadAll(reader)
	if err != nil {
		panic(fmt.Errorf("reading reader: %w", err))
	}
	return string(b)
}

func (FfiConverterstring) read(reader io.Reader) string {
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

func (FfiConverterstring) lower(value string) C.RustBuffer {
	return stringToCRustBuffer(value)
}

func (FfiConverterstring) write(writer io.Writer, value string) {
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

type FfiDestroyerstring struct{}

func (FfiDestroyerstring) destroy(_ string) {}

type FfiConverterBytes struct{}

var FfiConverterBytesINSTANCE = FfiConverterBytes{}

func (c FfiConverterBytes) lower(value []byte) C.RustBuffer {
	return goBytesToCRustBuffer(value)
}

func (c FfiConverterBytes) write(writer io.Writer, value []byte) {
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

func (c FfiConverterBytes) lift(value C.RustBuffer) []byte {
	reader := fromCRustBuffer(value).asReader()
	b, err := io.ReadAll(reader)
	if err != nil {
		panic(fmt.Errorf("reading reader: %w", err))
	}
	return b
}

func (c FfiConverterBytes) read(reader io.Reader) []byte {
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

func (FfiDestroyerBytes) destroy(_ []byte) {}

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

	return FfiConverterstringINSTANCE.lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
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

func (c FfiConverterAuthorId) lift(pointer unsafe.Pointer) *AuthorId {
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

func (c FfiConverterAuthorId) read(reader io.Reader) *AuthorId {
	return c.lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterAuthorId) lower(value *AuthorId) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*AuthorId")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterAuthorId) write(writer io.Writer, value *AuthorId) {
	writeUint64(writer, uint64(uintptr(c.lower(value))))
}

type FfiDestroyerAuthorId struct{}

func (_ FfiDestroyerAuthorId) destroy(value *AuthorId) {
	value.Destroy()
}

type Doc struct {
	ffiObject FfiObject
}

func (_self *Doc) GetContentBytes(entry *Entry) ([]byte, error) {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
		return C.uniffi_iroh_fn_method_doc_get_content_bytes(
			_pointer, FfiConverterEntryINSTANCE.lower(entry), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue []byte
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterBytesINSTANCE.lift(_uniffiRV), _uniffiErr
	}

}
func (_self *Doc) Id() string {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterstringINSTANCE.lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
		return C.uniffi_iroh_fn_method_doc_id(
			_pointer, _uniffiStatus)
	}))

}
func (_self *Doc) Keys() ([]*Entry, error) {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
		return C.uniffi_iroh_fn_method_doc_keys(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue []*Entry
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterSequenceEntryINSTANCE.lift(_uniffiRV), _uniffiErr
	}

}
func (_self *Doc) SetBytes(author *AuthorId, key []byte, value []byte) (*Entry, error) {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_doc_set_bytes(
			_pointer, FfiConverterAuthorIdINSTANCE.lower(author), FfiConverterBytesINSTANCE.lower(key), FfiConverterBytesINSTANCE.lower(value), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *Entry
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterEntryINSTANCE.lift(_uniffiRV), _uniffiErr
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
		return FfiConverterDocTicketINSTANCE.lift(_uniffiRV), _uniffiErr
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
		return FfiConverterDocTicketINSTANCE.lift(_uniffiRV), _uniffiErr
	}

}
func (_self *Doc) Status() (LiveStatus, error) {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
		return C.uniffi_iroh_fn_method_doc_status(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue LiveStatus
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterTypeLiveStatusINSTANCE.lift(_uniffiRV), _uniffiErr
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
func (_self *Doc) Subscribe(cb SubscribeCallback) error {
	_pointer := _self.ffiObject.incrementPointer("*Doc")
	defer _self.ffiObject.decrementPointer()

	_, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_iroh_fn_method_doc_subscribe(
			_pointer, FfiConverterTypeSubscribeCallbackINSTANCE.lower(cb), _uniffiStatus)
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

func (c FfiConverterDoc) lift(pointer unsafe.Pointer) *Doc {
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

func (c FfiConverterDoc) read(reader io.Reader) *Doc {
	return c.lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterDoc) lower(value *Doc) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*Doc")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterDoc) write(writer io.Writer, value *Doc) {
	writeUint64(writer, uint64(uintptr(c.lower(value))))
}

type FfiDestroyerDoc struct{}

func (_ FfiDestroyerDoc) destroy(value *Doc) {
	value.Destroy()
}

type DocTicket struct {
	ffiObject FfiObject
}

func DocTicketFromString(content string) (*DocTicket, error) {

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_constructor_docticket_from_string(FfiConverterstringINSTANCE.lower(content), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *DocTicket
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDocTicketINSTANCE.lift(_uniffiRV), _uniffiErr
	}

}

func (_self *DocTicket) ToString() string {
	_pointer := _self.ffiObject.incrementPointer("*DocTicket")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterstringINSTANCE.lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
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

func (c FfiConverterDocTicket) lift(pointer unsafe.Pointer) *DocTicket {
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

func (c FfiConverterDocTicket) read(reader io.Reader) *DocTicket {
	return c.lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterDocTicket) lower(value *DocTicket) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*DocTicket")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterDocTicket) write(writer io.Writer, value *DocTicket) {
	writeUint64(writer, uint64(uintptr(c.lower(value))))
}

type FfiDestroyerDocTicket struct{}

func (_ FfiDestroyerDocTicket) destroy(value *DocTicket) {
	value.Destroy()
}

type Entry struct {
	ffiObject FfiObject
}

func (_self *Entry) Author() *AuthorId {
	_pointer := _self.ffiObject.incrementPointer("*Entry")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterAuthorIdINSTANCE.lift(rustCall(func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_entry_author(
			_pointer, _uniffiStatus)
	}))

}
func (_self *Entry) Hash() *Hash {
	_pointer := _self.ffiObject.incrementPointer("*Entry")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterHashINSTANCE.lift(rustCall(func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_entry_hash(
			_pointer, _uniffiStatus)
	}))

}
func (_self *Entry) Key() []byte {
	_pointer := _self.ffiObject.incrementPointer("*Entry")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterBytesINSTANCE.lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
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

func (c FfiConverterEntry) lift(pointer unsafe.Pointer) *Entry {
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

func (c FfiConverterEntry) read(reader io.Reader) *Entry {
	return c.lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterEntry) lower(value *Entry) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*Entry")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterEntry) write(writer io.Writer, value *Entry) {
	writeUint64(writer, uint64(uintptr(c.lower(value))))
}

type FfiDestroyerEntry struct{}

func (_ FfiDestroyerEntry) destroy(value *Entry) {
	value.Destroy()
}

type Hash struct {
	ffiObject FfiObject
}

func (_self *Hash) ToBytes() []byte {
	_pointer := _self.ffiObject.incrementPointer("*Hash")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterBytesINSTANCE.lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
		return C.uniffi_iroh_fn_method_hash_to_bytes(
			_pointer, _uniffiStatus)
	}))

}
func (_self *Hash) ToString() string {
	_pointer := _self.ffiObject.incrementPointer("*Hash")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterstringINSTANCE.lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
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

func (c FfiConverterHash) lift(pointer unsafe.Pointer) *Hash {
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

func (c FfiConverterHash) read(reader io.Reader) *Hash {
	return c.lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterHash) lower(value *Hash) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*Hash")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterHash) write(writer io.Writer, value *Hash) {
	writeUint64(writer, uint64(uintptr(c.lower(value))))
}

type FfiDestroyerHash struct{}

func (_ FfiDestroyerHash) destroy(value *Hash) {
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
		return FfiConverterIrohNodeINSTANCE.lift(_uniffiRV), _uniffiErr
	}

}

func (_self *IrohNode) AuthorList() ([]*AuthorId, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
		return C.uniffi_iroh_fn_method_irohnode_author_list(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue []*AuthorId
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterSequenceAuthorIdINSTANCE.lift(_uniffiRV), _uniffiErr
	}

}
func (_self *IrohNode) AuthorNew() (*AuthorId, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_irohnode_author_new(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *AuthorId
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterAuthorIdINSTANCE.lift(_uniffiRV), _uniffiErr
	}

}
func (_self *IrohNode) ConnectionInfo(nodeId *PublicKey) (*ConnectionInfo, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
		return C.uniffi_iroh_fn_method_irohnode_connection_info(
			_pointer, FfiConverterPublicKeyINSTANCE.lower(nodeId), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *ConnectionInfo
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterOptionalTypeConnectionInfoINSTANCE.lift(_uniffiRV), _uniffiErr
	}

}
func (_self *IrohNode) Connections() ([]ConnectionInfo, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
		return C.uniffi_iroh_fn_method_irohnode_connections(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue []ConnectionInfo
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterSequenceTypeConnectionInfoINSTANCE.lift(_uniffiRV), _uniffiErr
	}

}
func (_self *IrohNode) DocJoin(ticket *DocTicket) (*Doc, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_irohnode_doc_join(
			_pointer, FfiConverterDocTicketINSTANCE.lower(ticket), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *Doc
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDocINSTANCE.lift(_uniffiRV), _uniffiErr
	}

}
func (_self *IrohNode) DocNew() (*Doc, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) unsafe.Pointer {
		return C.uniffi_iroh_fn_method_irohnode_doc_new(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *Doc
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDocINSTANCE.lift(_uniffiRV), _uniffiErr
	}

}
func (_self *IrohNode) NodeId() string {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterstringINSTANCE.lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
		return C.uniffi_iroh_fn_method_irohnode_node_id(
			_pointer, _uniffiStatus)
	}))

}
func (_self *IrohNode) Stats() (map[string]CounterStats, error) {
	_pointer := _self.ffiObject.incrementPointer("*IrohNode")
	defer _self.ffiObject.decrementPointer()

	_uniffiRV, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
		return C.uniffi_iroh_fn_method_irohnode_stats(
			_pointer, _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue map[string]CounterStats
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterMapstringTypeCounterStatsINSTANCE.lift(_uniffiRV), _uniffiErr
	}

}

func (object *IrohNode) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterIrohNode struct{}

var FfiConverterIrohNodeINSTANCE = FfiConverterIrohNode{}

func (c FfiConverterIrohNode) lift(pointer unsafe.Pointer) *IrohNode {
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

func (c FfiConverterIrohNode) read(reader io.Reader) *IrohNode {
	return c.lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterIrohNode) lower(value *IrohNode) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*IrohNode")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterIrohNode) write(writer io.Writer, value *IrohNode) {
	writeUint64(writer, uint64(uintptr(c.lower(value))))
}

type FfiDestroyerIrohNode struct{}

func (_ FfiDestroyerIrohNode) destroy(value *IrohNode) {
	value.Destroy()
}

type PublicKey struct {
	ffiObject FfiObject
}

func (_self *PublicKey) ToBytes() []byte {
	_pointer := _self.ffiObject.incrementPointer("*PublicKey")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterBytesINSTANCE.lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
		return C.uniffi_iroh_fn_method_publickey_to_bytes(
			_pointer, _uniffiStatus)
	}))

}
func (_self *PublicKey) ToString() string {
	_pointer := _self.ffiObject.incrementPointer("*PublicKey")
	defer _self.ffiObject.decrementPointer()

	return FfiConverterstringINSTANCE.lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.RustBuffer {
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

func (c FfiConverterPublicKey) lift(pointer unsafe.Pointer) *PublicKey {
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

func (c FfiConverterPublicKey) read(reader io.Reader) *PublicKey {
	return c.lift(unsafe.Pointer(uintptr(readUint64(reader))))
}

func (c FfiConverterPublicKey) lower(value *PublicKey) unsafe.Pointer {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the pointer will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked pointer.
	pointer := value.ffiObject.incrementPointer("*PublicKey")
	defer value.ffiObject.decrementPointer()
	return pointer
}

func (c FfiConverterPublicKey) write(writer io.Writer, value *PublicKey) {
	writeUint64(writer, uint64(uintptr(c.lower(value))))
}

type FfiDestroyerPublicKey struct{}

func (_ FfiDestroyerPublicKey) destroy(value *PublicKey) {
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
	FfiDestroyeruint64{}.destroy(r.Id)
	FfiDestroyerPublicKey{}.destroy(r.PublicKey)
	FfiDestroyerOptionaluint16{}.destroy(r.DerpRegion)
	FfiDestroyerSequenceTypeSocketAddr{}.destroy(r.Addrs)
	FfiDestroyerSequenceOptionalfloat64{}.destroy(r.Latencies)
	FfiDestroyerTypeConnectionType{}.destroy(r.ConnType)
	FfiDestroyerOptionalfloat64{}.destroy(r.Latency)
}

type FfiConverterTypeConnectionInfo struct{}

var FfiConverterTypeConnectionInfoINSTANCE = FfiConverterTypeConnectionInfo{}

func (c FfiConverterTypeConnectionInfo) lift(cRustBuf C.RustBuffer) ConnectionInfo {
	rustBuffer := fromCRustBuffer(cRustBuf)
	return liftFromRustBuffer[ConnectionInfo](c, rustBuffer)
}

func (c FfiConverterTypeConnectionInfo) read(reader io.Reader) ConnectionInfo {
	return ConnectionInfo{
		FfiConverteruint64INSTANCE.read(reader),
		FfiConverterPublicKeyINSTANCE.read(reader),
		FfiConverterOptionaluint16INSTANCE.read(reader),
		FfiConverterSequenceTypeSocketAddrINSTANCE.read(reader),
		FfiConverterSequenceOptionalfloat64INSTANCE.read(reader),
		FfiConverterTypeConnectionTypeINSTANCE.read(reader),
		FfiConverterOptionalfloat64INSTANCE.read(reader),
	}
}

func (c FfiConverterTypeConnectionInfo) lower(value ConnectionInfo) C.RustBuffer {
	return lowerIntoRustBuffer[ConnectionInfo](c, value)
}

func (c FfiConverterTypeConnectionInfo) write(writer io.Writer, value ConnectionInfo) {
	FfiConverteruint64INSTANCE.write(writer, value.Id)
	FfiConverterPublicKeyINSTANCE.write(writer, value.PublicKey)
	FfiConverterOptionaluint16INSTANCE.write(writer, value.DerpRegion)
	FfiConverterSequenceTypeSocketAddrINSTANCE.write(writer, value.Addrs)
	FfiConverterSequenceOptionalfloat64INSTANCE.write(writer, value.Latencies)
	FfiConverterTypeConnectionTypeINSTANCE.write(writer, value.ConnType)
	FfiConverterOptionalfloat64INSTANCE.write(writer, value.Latency)
}

type FfiDestroyerTypeConnectionInfo struct{}

func (_ FfiDestroyerTypeConnectionInfo) destroy(value ConnectionInfo) {
	value.Destroy()
}

type CounterStats struct {
	Value       uint64
	Description string
}

func (r *CounterStats) Destroy() {
	FfiDestroyeruint64{}.destroy(r.Value)
	FfiDestroyerstring{}.destroy(r.Description)
}

type FfiConverterTypeCounterStats struct{}

var FfiConverterTypeCounterStatsINSTANCE = FfiConverterTypeCounterStats{}

func (c FfiConverterTypeCounterStats) lift(cRustBuf C.RustBuffer) CounterStats {
	rustBuffer := fromCRustBuffer(cRustBuf)
	return liftFromRustBuffer[CounterStats](c, rustBuffer)
}

func (c FfiConverterTypeCounterStats) read(reader io.Reader) CounterStats {
	return CounterStats{
		FfiConverteruint64INSTANCE.read(reader),
		FfiConverterstringINSTANCE.read(reader),
	}
}

func (c FfiConverterTypeCounterStats) lower(value CounterStats) C.RustBuffer {
	return lowerIntoRustBuffer[CounterStats](c, value)
}

func (c FfiConverterTypeCounterStats) write(writer io.Writer, value CounterStats) {
	FfiConverteruint64INSTANCE.write(writer, value.Value)
	FfiConverterstringINSTANCE.write(writer, value.Description)
}

type FfiDestroyerTypeCounterStats struct{}

func (_ FfiDestroyerTypeCounterStats) destroy(value CounterStats) {
	value.Destroy()
}

type LiveStatus struct {
	Active        bool
	Subscriptions uint64
}

func (r *LiveStatus) Destroy() {
	FfiDestroyerbool{}.destroy(r.Active)
	FfiDestroyeruint64{}.destroy(r.Subscriptions)
}

type FfiConverterTypeLiveStatus struct{}

var FfiConverterTypeLiveStatusINSTANCE = FfiConverterTypeLiveStatus{}

func (c FfiConverterTypeLiveStatus) lift(cRustBuf C.RustBuffer) LiveStatus {
	rustBuffer := fromCRustBuffer(cRustBuf)
	return liftFromRustBuffer[LiveStatus](c, rustBuffer)
}

func (c FfiConverterTypeLiveStatus) read(reader io.Reader) LiveStatus {
	return LiveStatus{
		FfiConverterboolINSTANCE.read(reader),
		FfiConverteruint64INSTANCE.read(reader),
	}
}

func (c FfiConverterTypeLiveStatus) lower(value LiveStatus) C.RustBuffer {
	return lowerIntoRustBuffer[LiveStatus](c, value)
}

func (c FfiConverterTypeLiveStatus) write(writer io.Writer, value LiveStatus) {
	FfiConverterboolINSTANCE.write(writer, value.Active)
	FfiConverteruint64INSTANCE.write(writer, value.Subscriptions)
}

type FfiDestroyerTypeLiveStatus struct{}

func (_ FfiDestroyerTypeLiveStatus) destroy(value LiveStatus) {
	value.Destroy()
}

type ConnectionType interface {
	Destroy()
}
type ConnectionTypeDirect struct {
	Addr SocketAddr
}

func (e ConnectionTypeDirect) Destroy() {
	FfiDestroyerTypeSocketAddr{}.destroy(e.Addr)
}

type ConnectionTypeRelay struct {
	Port uint16
}

func (e ConnectionTypeRelay) Destroy() {
	FfiDestroyeruint16{}.destroy(e.Port)
}

type ConnectionTypeNone struct {
}

func (e ConnectionTypeNone) Destroy() {
}

type FfiConverterTypeConnectionType struct{}

var FfiConverterTypeConnectionTypeINSTANCE = FfiConverterTypeConnectionType{}

func (c FfiConverterTypeConnectionType) lift(cRustBuf C.RustBuffer) ConnectionType {
	return liftFromRustBuffer[ConnectionType](c, fromCRustBuffer(cRustBuf))
}

func (c FfiConverterTypeConnectionType) lower(value ConnectionType) C.RustBuffer {
	return lowerIntoRustBuffer[ConnectionType](c, value)
}
func (FfiConverterTypeConnectionType) read(reader io.Reader) ConnectionType {
	id := readInt32(reader)
	switch id {
	case 1:
		return ConnectionTypeDirect{
			FfiConverterTypeSocketAddrINSTANCE.read(reader),
		}
	case 2:
		return ConnectionTypeRelay{
			FfiConverteruint16INSTANCE.read(reader),
		}
	case 3:
		return ConnectionTypeNone{}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterTypeConnectionType.read()", id))
	}
}

func (FfiConverterTypeConnectionType) write(writer io.Writer, value ConnectionType) {
	switch variant_value := value.(type) {
	case ConnectionTypeDirect:
		writeInt32(writer, 1)
		FfiConverterTypeSocketAddrINSTANCE.write(writer, variant_value.Addr)
	case ConnectionTypeRelay:
		writeInt32(writer, 2)
		FfiConverteruint16INSTANCE.write(writer, variant_value.Port)
	case ConnectionTypeNone:
		writeInt32(writer, 3)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterTypeConnectionType.write", value))
	}
}

type FfiDestroyerTypeConnectionType struct{}

func (_ FfiDestroyerTypeConnectionType) destroy(value ConnectionType) {
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

func (c FfiConverterTypeIrohError) lift(cErrBuf C.RustBuffer) error {
	errBuf := fromCRustBuffer(cErrBuf)
	return liftFromRustBuffer[error](c, errBuf)
}

func (c FfiConverterTypeIrohError) lower(value *IrohError) C.RustBuffer {
	return lowerIntoRustBuffer[*IrohError](c, value)
}

func (c FfiConverterTypeIrohError) read(reader io.Reader) error {
	errorID := readUint32(reader)

	switch errorID {
	case 1:
		return &IrohError{&IrohErrorRuntime{
			Description: FfiConverterstringINSTANCE.read(reader),
		}}
	case 2:
		return &IrohError{&IrohErrorNodeCreate{
			Description: FfiConverterstringINSTANCE.read(reader),
		}}
	case 3:
		return &IrohError{&IrohErrorDoc{
			Description: FfiConverterstringINSTANCE.read(reader),
		}}
	case 4:
		return &IrohError{&IrohErrorAuthor{
			Description: FfiConverterstringINSTANCE.read(reader),
		}}
	case 5:
		return &IrohError{&IrohErrorDocTicket{
			Description: FfiConverterstringINSTANCE.read(reader),
		}}
	case 6:
		return &IrohError{&IrohErrorUniffi{
			Description: FfiConverterstringINSTANCE.read(reader),
		}}
	case 7:
		return &IrohError{&IrohErrorConnection{
			Description: FfiConverterstringINSTANCE.read(reader),
		}}
	default:
		panic(fmt.Sprintf("Unknown error code %d in FfiConverterTypeIrohError.read()", errorID))
	}
}

func (c FfiConverterTypeIrohError) write(writer io.Writer, value *IrohError) {
	switch variantValue := value.err.(type) {
	case *IrohErrorRuntime:
		writeInt32(writer, 1)
		FfiConverterstringINSTANCE.write(writer, variantValue.Description)
	case *IrohErrorNodeCreate:
		writeInt32(writer, 2)
		FfiConverterstringINSTANCE.write(writer, variantValue.Description)
	case *IrohErrorDoc:
		writeInt32(writer, 3)
		FfiConverterstringINSTANCE.write(writer, variantValue.Description)
	case *IrohErrorAuthor:
		writeInt32(writer, 4)
		FfiConverterstringINSTANCE.write(writer, variantValue.Description)
	case *IrohErrorDocTicket:
		writeInt32(writer, 5)
		FfiConverterstringINSTANCE.write(writer, variantValue.Description)
	case *IrohErrorUniffi:
		writeInt32(writer, 6)
		FfiConverterstringINSTANCE.write(writer, variantValue.Description)
	case *IrohErrorConnection:
		writeInt32(writer, 7)
		FfiConverterstringINSTANCE.write(writer, variantValue.Description)
	default:
		_ = variantValue
		panic(fmt.Sprintf("invalid error value `%v` in FfiConverterTypeIrohError.write", value))
	}
}

type LiveEvent uint

const (
	LiveEventInsertLocal  LiveEvent = 1
	LiveEventInsertRemote LiveEvent = 2
	LiveEventContentReady LiveEvent = 3
	LiveEventSyncFinished LiveEvent = 4
	LiveEventNeighborUp   LiveEvent = 5
	LiveEventNeighborDown LiveEvent = 6
)

type FfiConverterTypeLiveEvent struct{}

var FfiConverterTypeLiveEventINSTANCE = FfiConverterTypeLiveEvent{}

func (c FfiConverterTypeLiveEvent) lift(cRustBuf C.RustBuffer) LiveEvent {
	return liftFromRustBuffer[LiveEvent](c, fromCRustBuffer(cRustBuf))
}

func (c FfiConverterTypeLiveEvent) lower(value LiveEvent) C.RustBuffer {
	return lowerIntoRustBuffer[LiveEvent](c, value)
}
func (FfiConverterTypeLiveEvent) read(reader io.Reader) LiveEvent {
	id := readInt32(reader)
	return LiveEvent(id)
}

func (FfiConverterTypeLiveEvent) write(writer io.Writer, value LiveEvent) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerTypeLiveEvent struct{}

func (_ FfiDestroyerTypeLiveEvent) destroy(value LiveEvent) {
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

func (c FfiConverterTypeLogLevel) lift(cRustBuf C.RustBuffer) LogLevel {
	return liftFromRustBuffer[LogLevel](c, fromCRustBuffer(cRustBuf))
}

func (c FfiConverterTypeLogLevel) lower(value LogLevel) C.RustBuffer {
	return lowerIntoRustBuffer[LogLevel](c, value)
}
func (FfiConverterTypeLogLevel) read(reader io.Reader) LogLevel {
	id := readInt32(reader)
	return LogLevel(id)
}

func (FfiConverterTypeLogLevel) write(writer io.Writer, value LogLevel) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerTypeLogLevel struct{}

func (_ FfiDestroyerTypeLogLevel) destroy(value LogLevel) {
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
	FfiDestroyeruint8{}.destroy(e.A)
	FfiDestroyeruint8{}.destroy(e.B)
	FfiDestroyeruint8{}.destroy(e.C)
	FfiDestroyeruint8{}.destroy(e.D)
}

type SocketAddrV6 struct {
	Addr []byte
}

func (e SocketAddrV6) Destroy() {
	FfiDestroyerBytes{}.destroy(e.Addr)
}

type FfiConverterTypeSocketAddr struct{}

var FfiConverterTypeSocketAddrINSTANCE = FfiConverterTypeSocketAddr{}

func (c FfiConverterTypeSocketAddr) lift(cRustBuf C.RustBuffer) SocketAddr {
	return liftFromRustBuffer[SocketAddr](c, fromCRustBuffer(cRustBuf))
}

func (c FfiConverterTypeSocketAddr) lower(value SocketAddr) C.RustBuffer {
	return lowerIntoRustBuffer[SocketAddr](c, value)
}
func (FfiConverterTypeSocketAddr) read(reader io.Reader) SocketAddr {
	id := readInt32(reader)
	switch id {
	case 1:
		return SocketAddrV4{
			FfiConverteruint8INSTANCE.read(reader),
			FfiConverteruint8INSTANCE.read(reader),
			FfiConverteruint8INSTANCE.read(reader),
			FfiConverteruint8INSTANCE.read(reader),
		}
	case 2:
		return SocketAddrV6{
			FfiConverterBytesINSTANCE.read(reader),
		}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterTypeSocketAddr.read()", id))
	}
}

func (FfiConverterTypeSocketAddr) write(writer io.Writer, value SocketAddr) {
	switch variant_value := value.(type) {
	case SocketAddrV4:
		writeInt32(writer, 1)
		FfiConverteruint8INSTANCE.write(writer, variant_value.A)
		FfiConverteruint8INSTANCE.write(writer, variant_value.B)
		FfiConverteruint8INSTANCE.write(writer, variant_value.C)
		FfiConverteruint8INSTANCE.write(writer, variant_value.D)
	case SocketAddrV6:
		writeInt32(writer, 2)
		FfiConverterBytesINSTANCE.write(writer, variant_value.Addr)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterTypeSocketAddr.write", value))
	}
}

type FfiDestroyerTypeSocketAddr struct{}

func (_ FfiDestroyerTypeSocketAddr) destroy(value SocketAddr) {
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

func (c *FfiConverterCallbackInterface[CallbackInterface]) drop(handle uint64) rustBuffer {
	c.handleMap.remove(handle)
	return rustBuffer{}
}

func (c *FfiConverterCallbackInterface[CallbackInterface]) lift(handle uint64) CallbackInterface {
	val, ok := c.handleMap.tryGet(handle)
	if !ok {
		panic(fmt.Errorf("no callback in handle map: %d", handle))
	}
	return *val
}

func (c *FfiConverterCallbackInterface[CallbackInterface]) read(reader io.Reader) CallbackInterface {
	return c.lift(readUint64(reader))
}

func (c *FfiConverterCallbackInterface[CallbackInterface]) lower(value CallbackInterface) C.uint64_t {
	return C.uint64_t(c.handleMap.insert(&value))
}

func (c *FfiConverterCallbackInterface[CallbackInterface]) write(writer io.Writer, value CallbackInterface) {
	writeUint64(writer, uint64(c.lower(value)))
}

// Declaration and FfiConverters for SubscribeCallback Callback Interface
type SubscribeCallback interface {
	Event(event LiveEvent) *IrohError
}

// foreignCallbackTypeSubscribeCallback cannot be callable be a compiled function at a same time
type foreignCallbackTypeSubscribeCallback struct{}

//export iroh_cgo_SubscribeCallback
func iroh_cgo_SubscribeCallback(handle C.uint64_t, method C.int32_t, argsPtr *C.uint8_t, argsLen C.int32_t, outBuf *C.RustBuffer) C.int32_t {
	cb := FfiConverterTypeSubscribeCallbackINSTANCE.lift(uint64(handle))
	switch method {
	case 0:
		// 0 means Rust is done with the callback, and the callback
		// can be dropped by the foreign language.
		*outBuf = FfiConverterTypeSubscribeCallbackINSTANCE.drop(uint64(handle)).asCRustBuffer()
		// See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
		return C.int32_t(idxCallbackFree)

	case 1:
		var result uniffiCallbackResult
		args := unsafe.Slice((*byte)(argsPtr), argsLen)
		result = foreignCallbackTypeSubscribeCallback{}.InvokeEvent(cb, args, outBuf)
		return C.int32_t(result)

	default:
		// This should never happen, because an out of bounds method index won't
		// ever be used. Once we can catch errors, we should return an InternalException.
		// https://github.com/mozilla/uniffi-rs/issues/351
		return C.int32_t(uniffiCallbackUnexpectedResultError)
	}
}

func (foreignCallbackTypeSubscribeCallback) InvokeEvent(callback SubscribeCallback, args []byte, outBuf *C.RustBuffer) uniffiCallbackResult {
	reader := bytes.NewReader(args)
	err := callback.Event(FfiConverterTypeLiveEventINSTANCE.read(reader))

	if err != nil {
		// The only way to bypass an unexpected error is to bypass pointer to an empty
		// instance of the error
		if err.err == nil {
			return uniffiCallbackUnexpectedResultError
		}
		*outBuf = lowerIntoRustBuffer[*IrohError](FfiConverterTypeIrohErrorINSTANCE, err)
		return uniffiCallbackResultError
	}
	return uniffiCallbackResultSuccess
}

type FfiConverterTypeSubscribeCallback struct {
	FfiConverterCallbackInterface[SubscribeCallback]
}

var FfiConverterTypeSubscribeCallbackINSTANCE = &FfiConverterTypeSubscribeCallback{
	FfiConverterCallbackInterface: FfiConverterCallbackInterface[SubscribeCallback]{
		handleMap: newConcurrentHandleMap[SubscribeCallback](),
	},
}

// This is a static function because only 1 instance is supported for registering
func (c *FfiConverterTypeSubscribeCallback) register() {
	rustCall(func(status *C.RustCallStatus) int32 {
		C.uniffi_iroh_fn_init_callback_subscribecallback(C.ForeignCallback(C.iroh_cgo_SubscribeCallback), status)
		return 0
	})
}

type FfiDestroyerTypeSubscribeCallback struct{}

func (FfiDestroyerTypeSubscribeCallback) destroy(value SubscribeCallback) {
}

type FfiConverterOptionaluint16 struct{}

var FfiConverterOptionaluint16INSTANCE = FfiConverterOptionaluint16{}

func (c FfiConverterOptionaluint16) lift(cRustBuf C.RustBuffer) *uint16 {
	return liftFromRustBuffer[*uint16](c, fromCRustBuffer(cRustBuf))
}

func (_ FfiConverterOptionaluint16) read(reader io.Reader) *uint16 {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverteruint16INSTANCE.read(reader)
	return &temp
}

func (c FfiConverterOptionaluint16) lower(value *uint16) C.RustBuffer {
	return lowerIntoRustBuffer[*uint16](c, value)
}

func (_ FfiConverterOptionaluint16) write(writer io.Writer, value *uint16) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverteruint16INSTANCE.write(writer, *value)
	}
}

type FfiDestroyerOptionaluint16 struct{}

func (_ FfiDestroyerOptionaluint16) destroy(value *uint16) {
	if value != nil {
		FfiDestroyeruint16{}.destroy(*value)
	}
}

type FfiConverterOptionalfloat64 struct{}

var FfiConverterOptionalfloat64INSTANCE = FfiConverterOptionalfloat64{}

func (c FfiConverterOptionalfloat64) lift(cRustBuf C.RustBuffer) *float64 {
	return liftFromRustBuffer[*float64](c, fromCRustBuffer(cRustBuf))
}

func (_ FfiConverterOptionalfloat64) read(reader io.Reader) *float64 {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterfloat64INSTANCE.read(reader)
	return &temp
}

func (c FfiConverterOptionalfloat64) lower(value *float64) C.RustBuffer {
	return lowerIntoRustBuffer[*float64](c, value)
}

func (_ FfiConverterOptionalfloat64) write(writer io.Writer, value *float64) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterfloat64INSTANCE.write(writer, *value)
	}
}

type FfiDestroyerOptionalfloat64 struct{}

func (_ FfiDestroyerOptionalfloat64) destroy(value *float64) {
	if value != nil {
		FfiDestroyerfloat64{}.destroy(*value)
	}
}

type FfiConverterOptionalTypeConnectionInfo struct{}

var FfiConverterOptionalTypeConnectionInfoINSTANCE = FfiConverterOptionalTypeConnectionInfo{}

func (c FfiConverterOptionalTypeConnectionInfo) lift(cRustBuf C.RustBuffer) *ConnectionInfo {
	return liftFromRustBuffer[*ConnectionInfo](c, fromCRustBuffer(cRustBuf))
}

func (_ FfiConverterOptionalTypeConnectionInfo) read(reader io.Reader) *ConnectionInfo {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterTypeConnectionInfoINSTANCE.read(reader)
	return &temp
}

func (c FfiConverterOptionalTypeConnectionInfo) lower(value *ConnectionInfo) C.RustBuffer {
	return lowerIntoRustBuffer[*ConnectionInfo](c, value)
}

func (_ FfiConverterOptionalTypeConnectionInfo) write(writer io.Writer, value *ConnectionInfo) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterTypeConnectionInfoINSTANCE.write(writer, *value)
	}
}

type FfiDestroyerOptionalTypeConnectionInfo struct{}

func (_ FfiDestroyerOptionalTypeConnectionInfo) destroy(value *ConnectionInfo) {
	if value != nil {
		FfiDestroyerTypeConnectionInfo{}.destroy(*value)
	}
}

type FfiConverterSequenceAuthorId struct{}

var FfiConverterSequenceAuthorIdINSTANCE = FfiConverterSequenceAuthorId{}

func (c FfiConverterSequenceAuthorId) lift(cRustBuf C.RustBuffer) []*AuthorId {
	return liftFromRustBuffer[[]*AuthorId](c, fromCRustBuffer(cRustBuf))
}

func (c FfiConverterSequenceAuthorId) read(reader io.Reader) []*AuthorId {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]*AuthorId, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterAuthorIdINSTANCE.read(reader))
	}
	return result
}

func (c FfiConverterSequenceAuthorId) lower(value []*AuthorId) C.RustBuffer {
	return lowerIntoRustBuffer[[]*AuthorId](c, value)
}

func (c FfiConverterSequenceAuthorId) write(writer io.Writer, value []*AuthorId) {
	if len(value) > math.MaxInt32 {
		panic("[]*AuthorId is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterAuthorIdINSTANCE.write(writer, item)
	}
}

type FfiDestroyerSequenceAuthorId struct{}

func (FfiDestroyerSequenceAuthorId) destroy(sequence []*AuthorId) {
	for _, value := range sequence {
		FfiDestroyerAuthorId{}.destroy(value)
	}
}

type FfiConverterSequenceEntry struct{}

var FfiConverterSequenceEntryINSTANCE = FfiConverterSequenceEntry{}

func (c FfiConverterSequenceEntry) lift(cRustBuf C.RustBuffer) []*Entry {
	return liftFromRustBuffer[[]*Entry](c, fromCRustBuffer(cRustBuf))
}

func (c FfiConverterSequenceEntry) read(reader io.Reader) []*Entry {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]*Entry, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterEntryINSTANCE.read(reader))
	}
	return result
}

func (c FfiConverterSequenceEntry) lower(value []*Entry) C.RustBuffer {
	return lowerIntoRustBuffer[[]*Entry](c, value)
}

func (c FfiConverterSequenceEntry) write(writer io.Writer, value []*Entry) {
	if len(value) > math.MaxInt32 {
		panic("[]*Entry is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterEntryINSTANCE.write(writer, item)
	}
}

type FfiDestroyerSequenceEntry struct{}

func (FfiDestroyerSequenceEntry) destroy(sequence []*Entry) {
	for _, value := range sequence {
		FfiDestroyerEntry{}.destroy(value)
	}
}

type FfiConverterSequenceTypeConnectionInfo struct{}

var FfiConverterSequenceTypeConnectionInfoINSTANCE = FfiConverterSequenceTypeConnectionInfo{}

func (c FfiConverterSequenceTypeConnectionInfo) lift(cRustBuf C.RustBuffer) []ConnectionInfo {
	return liftFromRustBuffer[[]ConnectionInfo](c, fromCRustBuffer(cRustBuf))
}

func (c FfiConverterSequenceTypeConnectionInfo) read(reader io.Reader) []ConnectionInfo {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]ConnectionInfo, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterTypeConnectionInfoINSTANCE.read(reader))
	}
	return result
}

func (c FfiConverterSequenceTypeConnectionInfo) lower(value []ConnectionInfo) C.RustBuffer {
	return lowerIntoRustBuffer[[]ConnectionInfo](c, value)
}

func (c FfiConverterSequenceTypeConnectionInfo) write(writer io.Writer, value []ConnectionInfo) {
	if len(value) > math.MaxInt32 {
		panic("[]ConnectionInfo is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterTypeConnectionInfoINSTANCE.write(writer, item)
	}
}

type FfiDestroyerSequenceTypeConnectionInfo struct{}

func (FfiDestroyerSequenceTypeConnectionInfo) destroy(sequence []ConnectionInfo) {
	for _, value := range sequence {
		FfiDestroyerTypeConnectionInfo{}.destroy(value)
	}
}

type FfiConverterSequenceTypeSocketAddr struct{}

var FfiConverterSequenceTypeSocketAddrINSTANCE = FfiConverterSequenceTypeSocketAddr{}

func (c FfiConverterSequenceTypeSocketAddr) lift(cRustBuf C.RustBuffer) []SocketAddr {
	return liftFromRustBuffer[[]SocketAddr](c, fromCRustBuffer(cRustBuf))
}

func (c FfiConverterSequenceTypeSocketAddr) read(reader io.Reader) []SocketAddr {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]SocketAddr, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterTypeSocketAddrINSTANCE.read(reader))
	}
	return result
}

func (c FfiConverterSequenceTypeSocketAddr) lower(value []SocketAddr) C.RustBuffer {
	return lowerIntoRustBuffer[[]SocketAddr](c, value)
}

func (c FfiConverterSequenceTypeSocketAddr) write(writer io.Writer, value []SocketAddr) {
	if len(value) > math.MaxInt32 {
		panic("[]SocketAddr is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterTypeSocketAddrINSTANCE.write(writer, item)
	}
}

type FfiDestroyerSequenceTypeSocketAddr struct{}

func (FfiDestroyerSequenceTypeSocketAddr) destroy(sequence []SocketAddr) {
	for _, value := range sequence {
		FfiDestroyerTypeSocketAddr{}.destroy(value)
	}
}

type FfiConverterSequenceOptionalfloat64 struct{}

var FfiConverterSequenceOptionalfloat64INSTANCE = FfiConverterSequenceOptionalfloat64{}

func (c FfiConverterSequenceOptionalfloat64) lift(cRustBuf C.RustBuffer) []*float64 {
	return liftFromRustBuffer[[]*float64](c, fromCRustBuffer(cRustBuf))
}

func (c FfiConverterSequenceOptionalfloat64) read(reader io.Reader) []*float64 {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]*float64, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterOptionalfloat64INSTANCE.read(reader))
	}
	return result
}

func (c FfiConverterSequenceOptionalfloat64) lower(value []*float64) C.RustBuffer {
	return lowerIntoRustBuffer[[]*float64](c, value)
}

func (c FfiConverterSequenceOptionalfloat64) write(writer io.Writer, value []*float64) {
	if len(value) > math.MaxInt32 {
		panic("[]*float64 is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterOptionalfloat64INSTANCE.write(writer, item)
	}
}

type FfiDestroyerSequenceOptionalfloat64 struct{}

func (FfiDestroyerSequenceOptionalfloat64) destroy(sequence []*float64) {
	for _, value := range sequence {
		FfiDestroyerOptionalfloat64{}.destroy(value)
	}
}

type FfiConverterMapstringTypeCounterStats struct{}

var FfiConverterMapstringTypeCounterStatsINSTANCE = FfiConverterMapstringTypeCounterStats{}

func (c FfiConverterMapstringTypeCounterStats) lift(cRustBuf C.RustBuffer) map[string]CounterStats {
	rustBuffer := fromCRustBuffer(cRustBuf)
	return liftFromRustBuffer[map[string]CounterStats](c, rustBuffer)
}

func (_ FfiConverterMapstringTypeCounterStats) read(reader io.Reader) map[string]CounterStats {
	result := make(map[string]CounterStats)
	length := readInt32(reader)
	for i := int32(0); i < length; i++ {
		key := FfiConverterstringINSTANCE.read(reader)
		value := FfiConverterTypeCounterStatsINSTANCE.read(reader)
		result[key] = value
	}
	return result
}

func (c FfiConverterMapstringTypeCounterStats) lower(value map[string]CounterStats) C.RustBuffer {
	return lowerIntoRustBuffer[map[string]CounterStats](c, value)
}

func (_ FfiConverterMapstringTypeCounterStats) write(writer io.Writer, mapValue map[string]CounterStats) {
	if len(mapValue) > math.MaxInt32 {
		panic("map[string]CounterStats is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(mapValue)))
	for key, value := range mapValue {
		FfiConverterstringINSTANCE.write(writer, key)
		FfiConverterTypeCounterStatsINSTANCE.write(writer, value)
	}
}

type FfiDestroyerMapstringTypeCounterStats struct{}

func (_ FfiDestroyerMapstringTypeCounterStats) destroy(mapValue map[string]CounterStats) {
	for key, value := range mapValue {
		FfiDestroyerstring{}.destroy(key)
		FfiDestroyerTypeCounterStats{}.destroy(value)
	}
}

func SetLogLevel(level LogLevel) {

	rustCall(func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_iroh_fn_func_set_log_level(FfiConverterTypeLogLevelINSTANCE.lower(level), _uniffiStatus)
		return false
	})

}

func StartMetricsCollection() error {

	_, _uniffiErr := rustCallWithError(FfiConverterTypeIrohError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_iroh_fn_func_start_metrics_collection(_uniffiStatus)
		return false
	})
	return _uniffiErr

}
