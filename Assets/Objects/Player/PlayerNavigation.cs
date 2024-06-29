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

        activeWaypointIndicator.SetActive(true);
        activeWaypointIndicator.transform.position = agent.destination;

        if (!atDestination)
        {
            NavigationCancelledEvent.Invoke();
        }
        NavigationStartedEvent.Invoke();
        atDestination = false;
    }

    void Update()
    {
        if (!atDestination)
        {
            if(Mathf.Approximately(Vector3.Distance(agent.pathEndPosition, agent.transform.position), 0))
            {
                atDestination = true;
                NavigationFinishedEvent.Invoke();
                activeWaypointIndicator.SetActive(false);
            }
        }
    }
}
