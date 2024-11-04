using System.Collections;
using System.Collections.Generic;
using Cinemachine;
using UnityEngine;
using UnityEngine.AI;
using UnityEngine.InputSystem;

public class Player : SingletonMonoBehaviour<Player>
{
    [SerializeField] PlayerInput input;
    [SerializeField] PlayerNavigation navigation;
    [SerializeField] new Camera camera;
    [SerializeField] CinemachineVirtualCamera mainVCam;
    [SerializeField] CinemachineBrain cinemachineBrain;

    void Reset()
    {
        input = GetComponent<PlayerInput>();
        navigation = GetComponent<PlayerNavigation>();
        camera = FindObjectOfType<Camera>();
        mainVCam = FindObjectOfType<CinemachineVirtualCamera>();
        cinemachineBrain = FindObjectOfType<CinemachineBrain>();
    }

    public PlayerInput Input => input;
    public PlayerNavigation Navigation => navigation;
    public Camera Camera => camera;
    public CinemachineBrain CinemachineBrain => cinemachineBrain;
    public CinemachineVirtualCamera MainVCam => mainVCam;
}
