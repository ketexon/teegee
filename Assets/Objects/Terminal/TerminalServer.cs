using System;
using System.Net;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using UnityEngine.Serialization;

public enum MessageType : uint
{
    Initialize = 0,
    UnlockDoor = 1,
}

[StructLayout(LayoutKind.Sequential)]
public struct MessageHeader
{
    public MessageType Type;
    public uint Length;
}

public interface IMessage
{
    public MessageType Type { get; }
}

[Serializable]
public enum TerminalType : uint
{
    OS = 0,
    Pinpad = 1,
    Error = 37,
}

[Serializable]
[StructLayout(LayoutKind.Sequential)]
public struct InitializeMessage : IMessage
{
    public MessageType Type => MessageType.Initialize;

    [FormerlySerializedAs("Index")]
    public TerminalType TerminalType;
}

[Serializable]
[StructLayout(LayoutKind.Sequential)]
public struct UnlockDoorMessage : IMessage
{
    public MessageType Type => MessageType.UnlockDoor;

    [MarshalAs(UnmanagedType.ByValArray, SizeConst = 4)]
    public byte[] Code;
}

public class TerminalServer : IDisposable, IAsyncDisposable
{
    static IPAddress Address => IPAddress.Loopback;
    static int Port => 41987;

    static IPEndPoint EndPoint => new(Address, Port);

    TcpListener listener = null;
    TcpClient client = null;

    NetworkStream stream = null;

    public bool Connected => stream != null;

    public TerminalServer() {
        listener = new(EndPoint);
    }

    public void Start()
    {
        listener.Start();
    }

    public bool Pending => listener?.Pending() ?? false;

    byte[] DataToBytes<T>(T data)
        where T : struct
    {
        int size = Marshal.SizeOf<T>();
        IntPtr buffer = IntPtr.Zero;
        try
        {
            buffer = Marshal.AllocHGlobal(size);
            Marshal.StructureToPtr(data, buffer, false);
            byte[] rawData = new byte[size];
            Marshal.Copy(buffer, rawData, 0, size);
            return rawData;
        }
        finally
        {
            if (buffer != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(buffer);
            }
        }
    }

    T? BytesToData<T>(byte[] data)
    where T : struct
    {
        int size = Marshal.SizeOf<T>();
        if (size != data.Length)
        {
            return null;
        }

        IntPtr buffer = IntPtr.Zero;
        try
        {
            buffer = Marshal.AllocHGlobal(size);
            Marshal.Copy(data, 0, buffer, size);
            return Marshal.PtrToStructure<T>(buffer);
        }
        finally
        {
            if (buffer != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(buffer);
            }
        }
    }

    public void Write<T>(T data)
        where T : struct
    {
        Write(DataToBytes(data));
    }

    public void Write(ReadOnlySpan<byte> data)
    {
        stream.Write(data);
    }

    public async Task WriteAsync<T>(T data)
        where T : struct
    {
        await WriteAsync(DataToBytes(data));
    }

    public async Task WriteAsync(ReadOnlyMemory<byte> data)
    {
        await stream.WriteAsync(data);
    }

    public void WriteMessage<T>(T message)
        where T : struct, IMessage
    {
        Write(new MessageHeader
        {
            Type = message.Type,
            Length = (uint)Marshal.SizeOf<T>(),
        });

        Write(message);
    }

    public Task WriteMessageAsync<T>(T message)
        where T : struct, IMessage
    {
        Write(new MessageHeader
        {
            Type = message.Type,
            Length = (uint)Marshal.SizeOf<T>(),
        });

        return WriteAsync(message);
    }

    public T Read<T>()
        where T : struct
    {
        var data = ReadExactly(Marshal.SizeOf<T>());
        return BytesToData<T>(data).Value;
    }

    public async Task<T> ReadAsync<T>()
        where T : struct
    {
        var data = await ReadExactlyAsync(Marshal.SizeOf<T>());
        return BytesToData<T>(data).Value;
    }

    public byte[] ReadExactly(int n)
    {
        if (n == 0) return new byte[0];
        var buffer = new byte[n];
        int read = 0;
        int ToRead() => n - read;
        while (ToRead() > 0) {
            read += stream.Read(buffer, read, ToRead());
        }
        return buffer;
    }

    public async Task<byte[]> ReadExactlyAsync(int n)
    {
        if (n == 0) return new byte[0];
        var buffer = new byte[n];
        int read = 0;
        int ToRead() => n - read;
        while (ToRead() > 0)
        {
            read += await stream.ReadAsync(buffer, read, ToRead());
        }
        return buffer;
    }

    public async Task<IMessage> ReadMessageAsync()
    {
        var messageHeader = await ReadAsync<MessageHeader>();

        async Task<T?> Internal<T>()
            where T : struct
        {
            var len = messageHeader.Length;
            if (len != Marshal.SizeOf<T>()) return null;
            return await ReadAsync<T>();
        }
        
        switch (messageHeader.Type)
        {
            case MessageType.Initialize:
                return await Internal<InitializeMessage>();
            case MessageType.UnlockDoor:
                return await Internal<UnlockDoorMessage>();
            default:
                throw new NotImplementedException($"Unknown message sent: {messageHeader.Type}");
        }
    }

    #region Accept
    public async Task AcceptAsync()
    {
        try
        {
            client = await listener.AcceptTcpClientAsync();
            stream = client.GetStream();
        }
        catch (Exception)
        {
            if(stream is not null)
            {
                await stream.DisposeAsync();
                stream = null;
            }
            client?.Dispose();
            client = null;
            throw;
        }
    }

    public bool Accept()
    {
        try
        {
            if (!Pending) return false;
            client = listener.AcceptTcpClient();
            stream = client.GetStream();
            return true;
        }
        catch (Exception)
        {
            stream?.Dispose();
            stream = null;

            client?.Dispose();
            client = null;
            throw;
        }
    }
    #endregion

    #region Start/Stop
    public void Stop()
    {
        stream?.Dispose();
        stream = null;
        client?.Dispose();
        client = null;

        listener.Stop();
    }

    public async Task StopAsync()
    {
        if(stream is not null)
        {
            await stream.DisposeAsync();
        }
        stream = null;
        client?.Dispose();
        client = null;

        listener.Stop();
    }
    #endregion

    #region Disposal
    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    public async ValueTask DisposeAsync()
    {
        await DisposeAsyncCore();
        Dispose(false);
        GC.SuppressFinalize(this);
    }

    protected virtual void Dispose(bool disposing)
    {
        if (disposing)
        {
            stream?.Dispose();
            stream = null;

            client?.Dispose();
            client = null;

            listener.Stop();
            listener = null;
        }
    }

    protected virtual async ValueTask DisposeAsyncCore()
    {
        if (stream is not null)
        {
            await stream.DisposeAsync().ConfigureAwait(false);
        }
        stream = null;

        client?.Dispose();
        client = null;

        listener.Stop();
        listener = null;
    }
    #endregion
}