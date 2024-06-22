using System;
using System.Net;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Threading.Tasks;

public enum MessageType : uint
{
    Initialize = 0,
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

[StructLayout(LayoutKind.Sequential)]
public struct InitializeMessage : IMessage
{
    public MessageType Type => MessageType.Initialize;

    public int Index;
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