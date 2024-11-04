
using System.Runtime.InteropServices;
using System;
using UnityEngine.Serialization;

namespace IPC
{
    public enum MessageType : uint
    {
        Initialize = 0,
        InitializeOS = 4,
        UnlockDoor = 1,
        SwitchComputers = 2,
        PlaySfx = 3,
    }

    public enum ComputerID : uint
    {
        First = 0,
        Second = 1,
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
    public struct InitializeOSMessage : IMessage
    {
        public MessageType Type => MessageType.InitializeOS;

        public ComputerID ComputerID;
    }

    [Serializable]
    [StructLayout(LayoutKind.Sequential)]
    public struct UnlockDoorMessage : IMessage
    {
        public MessageType Type => MessageType.UnlockDoor;

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 4)]
        public byte[] Code;
    }

    [Serializable]
    [StructLayout(LayoutKind.Sequential)]
    public struct SwitchComputersMessage : IMessage
    {
        public MessageType Type => MessageType.SwitchComputers;

        public ComputerID NewID;
    }

    [Serializable]
    [StructLayout(LayoutKind.Sequential)]
    public struct PlaySfxMessage : IMessage
    {
        public MessageType Type => MessageType.PlaySfx;

        public uint ID;
    }
}