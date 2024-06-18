using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.AI;
using UnityEngine.InputSystem;

public class Player : SingletonMonoBehaviour<Player>
{
    [SerializeField] PlayerInput input;
    [SerializeField] PlayerNavigation navigation;
    [SerializeField] new Camera camera;

    void Reset()
    {
        input = GetComponent<PlayerInput>();
        navigation = GetComponent<PlayerNavigation>();
        camera = FindObjectOfType<Camera>();
    }

    public PlayerInput Input => input;
    public PlayerNavigation Navigation => navigation;
    public Camera Camera => camera;
}
