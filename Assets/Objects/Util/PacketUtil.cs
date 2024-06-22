using System.Runtime.InteropServices;
using System;

public static class PacketUtil
{
    public static byte[] SerializeBytes<T>(T t)
    {
        int size = Marshal.SizeOf<T>();
        byte[] arr = new byte[size];

        IntPtr p = IntPtr.Zero;
        try
        {
            p = Marshal.AllocHGlobal(size);
            Marshal.StructureToPtr(t, p, true);
            Marshal.Copy(p, arr, 0, size);
        }
        finally
        {
            Marshal.FreeHGlobal(p);
        }

        return arr;
    }

    public static T? DeserializeBytes<T>(byte[] arr)
        where T : struct
    {
        int size = Marshal.SizeOf<T>();
        if (size != arr.Length)
        {
            return null;
        }
        T? t = null;

        IntPtr p = IntPtr.Zero;
        try
        {
            p = Marshal.AllocHGlobal(size);
            Marshal.Copy(arr, 0, p, size);
            t = Marshal.PtrToStructure<T>(p);
        }
        finally
        {
            Marshal.FreeHGlobal(p);
        }

        return t;
    }
}