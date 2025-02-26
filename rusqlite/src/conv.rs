use stable_fs::error::Error;

use wasi_shim::wasi::{
    Errno, ERRNO_2BIG, ERRNO_ACCES, ERRNO_ADDRINUSE, ERRNO_ADDRNOTAVAIL, ERRNO_AFNOSUPPORT,
    ERRNO_AGAIN, ERRNO_ALREADY, ERRNO_BADF, ERRNO_BADMSG, ERRNO_BUSY, ERRNO_CANCELED, ERRNO_CHILD,
    ERRNO_CONNABORTED, ERRNO_CONNREFUSED, ERRNO_CONNRESET, ERRNO_DEADLK, ERRNO_DESTADDRREQ,
    ERRNO_DOM, ERRNO_DQUOT, ERRNO_EXIST, ERRNO_FAULT, ERRNO_FBIG, ERRNO_HOSTUNREACH, ERRNO_IDRM,
    ERRNO_ILSEQ, ERRNO_INPROGRESS, ERRNO_INTR, ERRNO_INVAL, ERRNO_IO, ERRNO_ISCONN, ERRNO_ISDIR,
    ERRNO_LOOP, ERRNO_MFILE, ERRNO_MLINK, ERRNO_MSGSIZE, ERRNO_MULTIHOP, ERRNO_NAMETOOLONG,
    ERRNO_NETDOWN, ERRNO_NETRESET, ERRNO_NETUNREACH, ERRNO_NFILE, ERRNO_NOBUFS, ERRNO_NODEV,
    ERRNO_NOENT, ERRNO_NOEXEC, ERRNO_NOLCK, ERRNO_NOLINK, ERRNO_NOMEM, ERRNO_NOMSG,
    ERRNO_NOPROTOOPT, ERRNO_NOSPC, ERRNO_NOSYS, ERRNO_NOTCAPABLE, ERRNO_NOTCONN, ERRNO_NOTDIR,
    ERRNO_NOTEMPTY, ERRNO_NOTRECOVERABLE, ERRNO_NOTSOCK, ERRNO_NOTSUP, ERRNO_NOTTY, ERRNO_NXIO,
    ERRNO_OVERFLOW, ERRNO_OWNERDEAD, ERRNO_PERM, ERRNO_PIPE, ERRNO_PROTO, ERRNO_PROTONOSUPPORT,
    ERRNO_PROTOTYPE, ERRNO_RANGE, ERRNO_ROFS, ERRNO_SPIPE, ERRNO_SRCH, ERRNO_STALE, ERRNO_TIMEDOUT,
    ERRNO_TXTBSY, ERRNO_XDEV,
};

pub fn error(err: Error) -> Errno {
    match err {
        Error::AddressFamilyNotSupported => ERRNO_AFNOSUPPORT,
        Error::AddressInUse => ERRNO_ADDRINUSE,
        Error::AddressNotAvailable => ERRNO_ADDRNOTAVAIL,
        Error::ArgumentListTooLong => ERRNO_2BIG,
        Error::BadAddress => ERRNO_FAULT,
        Error::BadFileDescriptor => ERRNO_BADF,
        Error::BadMessage => ERRNO_BADMSG,
        Error::BrokenPipe => ERRNO_PIPE,
        Error::ConnectionAborted => ERRNO_CONNABORTED,
        Error::ConnectionAbortedByNetwork => ERRNO_NETRESET,
        Error::ConnectionAlreadyInProgress => ERRNO_ALREADY,
        Error::ConnectionRefused => ERRNO_CONNREFUSED,
        Error::ConnectionReset => ERRNO_CONNRESET,
        Error::ConnectionTimedOut => ERRNO_TIMEDOUT,
        Error::CrossDeviceLink => ERRNO_XDEV,
        Error::DestinationAddressRequired => ERRNO_DESTADDRREQ,
        Error::DeviceOrResourceBusy => ERRNO_BUSY,
        Error::DirectoryNotEmpty => ERRNO_NOTEMPTY,
        Error::ExecutableFileFormatError => ERRNO_NOEXEC,
        Error::ExtensionCapabilitiesInsufficient => ERRNO_NOTCAPABLE,
        Error::FileDescriptorValueTooLarge => ERRNO_MFILE,
        Error::FileExists => ERRNO_EXIST,
        Error::FilenameTooLong => ERRNO_NAMETOOLONG,
        Error::FileTooLarge => ERRNO_FBIG,
        Error::FunctionNotSupported => ERRNO_NOSYS,
        Error::HostIsUnreachable => ERRNO_HOSTUNREACH,
        Error::IdentifierRemoved => ERRNO_IDRM,
        Error::IllegalByteSequence => ERRNO_ILSEQ,
        Error::InappropriateIOControlOperation => ERRNO_NOTTY,
        Error::InterruptedFunction => ERRNO_INTR,
        Error::InvalidArgument => ERRNO_INVAL,
        Error::InvalidSeek => ERRNO_SPIPE,
        Error::IOError => ERRNO_IO,
        Error::IsDirectory => ERRNO_ISDIR,
        Error::MathematicsArgumentOutOfDomainOfFunction => ERRNO_DOM,
        Error::MessageTooLarge => ERRNO_MSGSIZE,
        Error::NetworkIsDown => ERRNO_NETDOWN,
        Error::NetworkUnreachable => ERRNO_NETUNREACH,
        Error::NoBufferSpaceAvailable => ERRNO_NOBUFS,
        Error::NoChildProcesses => ERRNO_CHILD,
        Error::NoLocksAvailable => ERRNO_NOLCK,
        Error::NoMessageOfTheDesiredType => ERRNO_NOMSG,
        Error::NoSpaceLeftOnDevice => ERRNO_NOSPC,
        Error::NoSuchDevice => ERRNO_NODEV,
        Error::NoSuchDeviceOrAddress => ERRNO_NXIO,
        Error::NoSuchFileOrDirectory => ERRNO_NOENT,
        Error::NoSuchProcess => ERRNO_SRCH,
        Error::NotADirectoryOrSymbolicLink => ERRNO_NOTDIR,
        Error::NotASocket => ERRNO_NOTSOCK,
        Error::NotEnoughSpace => ERRNO_NOMEM,
        Error::NotSupportedOrOperationNotSupportedOnSocket => ERRNO_NOTSUP,
        Error::OperationCanceled => ERRNO_CANCELED,
        Error::OperationInProgress => ERRNO_INPROGRESS,
        Error::OperationNotPermitted => ERRNO_PERM,
        Error::PermissionDenied => ERRNO_ACCES,
        Error::PreviousOwnerDied => ERRNO_OWNERDEAD,
        Error::ProtocolError => ERRNO_PROTO,
        Error::ProtocolNotAvailable => ERRNO_NOPROTOOPT,
        Error::ProtocolNotSupported => ERRNO_PROTONOSUPPORT,
        Error::ProtocolWrongTypeForSocket => ERRNO_PROTOTYPE,
        Error::ReadOnlyFileSystem => ERRNO_ROFS,
        Error::Reserved19 => ERRNO_DQUOT,
        Error::Reserved36 => ERRNO_MULTIHOP,
        Error::Reserved47 => ERRNO_NOLINK,
        Error::Reserved72 => ERRNO_STALE,
        Error::ResourceDeadlockWouldOccur => ERRNO_DEADLK,
        Error::ResourceUnavailableOrOperationWouldBlock => ERRNO_AGAIN,
        Error::ResultTooLarge => ERRNO_RANGE,
        Error::SocketIsConnected => ERRNO_ISCONN,
        Error::SocketNotConnected => ERRNO_NOTCONN,
        Error::StateNotRecoverable => ERRNO_NOTRECOVERABLE,
        Error::TextFileBusy => ERRNO_TXTBSY,
        Error::TooManyFilesOpenInSystem => ERRNO_NFILE,
        Error::TooManyLevelsOfSymbolicLinks => ERRNO_LOOP,
        Error::TooManyLinks => ERRNO_MLINK,
        Error::ValueTooLargeToBeStoredInDataType => ERRNO_OVERFLOW,
    }
}
