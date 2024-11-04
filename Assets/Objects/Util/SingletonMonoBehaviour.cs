using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public enum SingletonDestroyMode {
    None,
    This,
    Other,
}

public class SingletonMonoBehaviour<T> : MonoBehaviour
    where T : SingletonMonoBehaviour<T>
{
    virtual protected SingletonDestroyMode DestroyMode => SingletonDestroyMode.This;
    virtual protected bool WarnOnMultiple => true;

    public static T Instance {  get; private set; }

    virtual protected void Awake()
    {
        if(Instance != null)
        {
            if(DestroyMode != SingletonDestroyMode.None) {
                Destroy(
                    DestroyMode == SingletonDestroyMode.This
                    ? this
                    : Instance
                );
            }
            if(WarnOnMultiple){
                Debug.LogWarning($"Warning: two instances of SingletonMonoBahviour {nameof(T)} founds.");
            }
        }

        Instance = this as T;
    }

    virtual protected void OnDestroy(){
        if(Instance == this){
            Instance = null;
        }
    }
}
