using System.Threading.Tasks;
using UnityEngine;

public class OS : Terminal
{
    [SerializeField]
    IPC.InitializeOSMessage initializeOSMessage;

    protected override void OnConnected()
    {
        base.OnConnected();

        Debug.Log($"Connected to OS. ComputerID: {initializeOSMessage.ComputerID}");
        TerminalMode.Instance.Server.WriteMessage(initializeOSMessage);
    }

    protected override void OnMessage(IPC.IMessage message)
    {
        base.OnMessage(message);

        if (message is IPC.SwitchComputersMessage switchComputersMessage)
        {
			RoomManager.Instance.LoadRoomAsync(switchComputersMessage.NewID).ContinueWith((task) => {
				Debug.Log(Room.Instance);
			});
        }
    }
}