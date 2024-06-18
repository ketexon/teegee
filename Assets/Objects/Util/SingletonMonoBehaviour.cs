using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class SingletonMonoBehaviour<T> : MonoBehaviour 
    where T : SingletonMonoBehaviour<T>
{
    public static T Instance {  get; private set; }

    virtual protected void Awake()
    {
        if(Instance != null)
        {
            Destroy(this);
            Debug.LogWarning("Warning: two instances of SingletonMonoBahviour founds. Deleting newer one.");
        }

        Instance = this as T;
    }
}
