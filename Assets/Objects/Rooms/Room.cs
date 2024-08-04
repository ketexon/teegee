using System.Collections;
using System.Collections.Generic;
using IPC;
using UnityEngine;

[System.Serializable]
public class RoomSpawn {
    public ComputerID Computer;
    public Transform Spawn;
}

public class Room : MonoBehaviour
{
    [SerializeField]
    public List<RoomSpawn> SpawnEntries;
}
