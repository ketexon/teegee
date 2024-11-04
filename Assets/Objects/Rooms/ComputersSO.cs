using System.Collections.Generic;
using UnityEngine;

[System.Serializable]
public class ComputersSOEntry {
	public IPC.ComputerID ComputerID;
	public string SceneName;
}

[CreateAssetMenu(fileName = "Computers", menuName = "Rooms/Computers")]
public class ComputersSO : SingletonScriptableObject<ComputersSO>
{
	[SerializeField]
	public List<ComputersSOEntry> Entries;

	public string FindSceneName(IPC.ComputerID id){
		return Entries.Find(entry => entry.ComputerID == id)?.SceneName;
	}
}
