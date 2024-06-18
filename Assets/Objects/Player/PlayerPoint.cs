using System;
using UnityEngine;
using UnityEngine.AI;
using UnityEngine.InputSystem;

public class PlayerPoint : MonoBehaviour
{
    [SerializeField] GameObject inactiveWaypointIndicator;
    [SerializeField, NavMeshMask] int walkableMask;
    [SerializeField] LayerMask interactableLayer;
    [SerializeField] float interactDistance = 1.0f;

    [SerializeField] InputActionReference pointAction;
    [SerializeField] InputActionReference clickAction;

    Vector2 mousePos = Vector2.zero;

    Vector3? navMeshDestination = null;
    Interactable interactable = null;
    Collider interactableCollider = null;

    void Awake()
    {
        inactiveWaypointIndicator.SetActive(false);
    }

    void OnEnable()
    {
        pointAction.action.performed += OnPoint;
        clickAction.action.performed += OnClick;
    }

    void OnDisable()
    {
        pointAction.action.performed -= OnPoint;
        clickAction.action.performed -= OnClick;
    }

    void OnPoint(InputAction.CallbackContext ctx)
    {
        mousePos = ctx.ReadValue<Vector2>();
    }

    void Update()
    {
        var ray = Player.Instance.Camera.ScreenPointToRay(mousePos);

        Vector3? newNavMeshDestination = null;
        Interactable newInteractable = null;

        if (Physics.Raycast(ray, out var hitInfo)) {
            // the hit is on a walkable part of the navmesh
            if(NavMesh.SamplePosition(hitInfo.point, out var _, .1f, walkableMask))
            {
                newNavMeshDestination = hitInfo.point;
            
                inactiveWaypointIndicator.SetActive(true);
                inactiveWaypointIndicator.transform.position = newNavMeshDestination.Value;
            }
            else
            {
                var goLayer = hitInfo.collider.gameObject.layer;
                if(interactableLayer.Contains(goLayer))
                {
                    newInteractable = hitInfo.collider.gameObject.GetComponent<Interactable>();
                }
            }
        }

        navMeshDestination = newNavMeshDestination;
        inactiveWaypointIndicator.SetActive(navMeshDestination != null);


        if(interactable != newInteractable)
        {
            if (interactable)
            {
                interactable.Unhighlight();
            }
            if (newInteractable)
            {
                newInteractable.Highlight();
            }
        }

        interactable = newInteractable;
        interactableCollider = interactable != null ? hitInfo.collider : null;
    }

    void OnClick(InputAction.CallbackContext ctx)
    {
        if(navMeshDestination != null)
        {
            Player.Instance.Navigation.NavigateTo(navMeshDestination.Value);
        }

        if (interactable)
        {
            Vector3 closest = Physics.ClosestPoint(
                transform.position, 
                interactableCollider,
                interactableCollider.transform.position,
                interactableCollider.transform.rotation
            );
            if (Vector3.Distance(closest, transform.position) > interactDistance) {
                Player.Instance.Navigation.NavigateTo(interactable.transform.position);
            }
            else
            {
                interactable.Interact();
            }
        }
    }
}
