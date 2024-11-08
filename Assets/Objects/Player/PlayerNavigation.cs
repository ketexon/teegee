using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.AI;
using UnityEngine.Events;
using UnityEngine.InputSystem;

public class PlayerNavigation : MonoBehaviour
{
    [SerializeField] GameObject activeWaypointIndicator;
    [SerializeField] NavMeshAgent agent;

    public UnityEvent NavigationStartedEvent;
    public UnityEvent NavigationCancelledEvent;
    public UnityEvent NavigationFinishedEvent;

    bool atDestination = true;
    bool waypointSet = false;

    void Reset()
    {
        agent = GetComponent<NavMeshAgent>();
    }

    void Awake()
    {
        activeWaypointIndicator.SetActive(false);
    }

    public void NavigateTo(Vector3 pos)
    {
        agent.destination = pos;

        if (!atDestination)
        {
            NavigationCancelledEvent.Invoke();
        }
        NavigationStartedEvent.Invoke();
        atDestination = false;
        waypointSet = false;
    }

    public void TeleportTo(Transform transform){
        agent.Warp(transform.position);
        transform.rotation = transform.rotation;
    }

    public void TeleportTo(IPC.ComputerID computerID){
        var spawn = Room.Instance.FindRoomSpawn(computerID);
        if(spawn == null){
            Debug.LogError($"Tried to teleport to computer with ID {computerID}, no spawn exists.");
        }
        else{
            TeleportTo(spawn);
        }
    }

    void Update()
    {
        if (!atDestination && !agent.pathPending)
        {
            if(!waypointSet) {
                activeWaypointIndicator.SetActive(true);
                activeWaypointIndicator.transform.position = agent.pathEndPosition;
                waypointSet = true;
            }

            if(Mathf.Approximately(Vector3.Distance(agent.pathEndPosition, agent.transform.position), 0))
            {
                atDestination = true;
                NavigationFinishedEvent.Invoke();
                activeWaypointIndicator.SetActive(false);
            }
        }
    }
}
