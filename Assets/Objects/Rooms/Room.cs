using System.Collections;
using System.Collections.Generic;
using IPC;
using UnityEngine;
using UnityEngine.Serialization;

[System.Serializable]
public class RoomSpawn {
    public ComputerID Computer;
    public Transform Spawn;
}

public class Room : SingletonMonoBehaviour<Room>
{
    [FormerlySerializedAs("SpawnEntries")]
    [SerializeField]
    public List<RoomSpawn> spawnEntries;

    public Transform FindRoomSpawn(ComputerID id){
        return spawnEntries.Find(entry => entry.Computer == id)?.Spawn;
    }
}
