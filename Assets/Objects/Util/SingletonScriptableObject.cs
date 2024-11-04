using UnityEngine;

public class SingletonScriptableObject<T> : ScriptableObject
	where T : SingletonScriptableObject<T>
{
	[System.NonSerialized]
	public static T Instance = null;

	protected virtual void OnEnable() {
		if(Instance && Instance != this) {
			Debug.LogWarning($"Multiple instances of {nameof(T)}");
			return;
		}
		Instance = this as T;
	}
}