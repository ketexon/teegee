using System;
using UnityEngine;
using UnityEngine.AI;
using UnityEngine.EventSystems;
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
        clickAction.action.canceled += OnClick;
    }

    void OnDisable()
    {
        pointAction.action.performed -= OnPoint;
        clickAction.action.canceled -= OnClick;
    }

    void OnPoint(InputAction.CallbackContext ctx)
    {
        mousePos = ctx.ReadValue<Vector2>();
    }

    void Update()
    {
        var ray = Player.Instance.Camera.ScreenPointToRay(mousePos);

        Interactable newInteractable = null;
        RaycastHit hitInfo = new();

        if (!EventSystem.current.IsPointerOverGameObject() && Physics.Raycast(ray, out hitInfo)) {
            var goLayer = hitInfo.collider.gameObject.layer;
            if (interactableLayer.Contains(goLayer))
            {
                newInteractable = hitInfo.collider.gameObject.GetComponent<Interactable>();
                navMeshDestination = null;
                inactiveWaypointIndicator.SetActive(false);
            }
            else {
                navMeshDestination = hitInfo.point;
                inactiveWaypointIndicator.SetActive(true);
                inactiveWaypointIndicator.transform.position = hitInfo.point;
                inactiveWaypointIndicator.transform.rotation = Quaternion.FromToRotation(Vector3.up, hitInfo.normal);
            }
            // this code can be used to determine if the
            // hit point is on a walkable part of the navmesh
            // else if (NavMesh.SamplePosition(hitInfo.point, out var _, .1f, walkableMask))
            // {
            //     navMeshDestination = hitInfo.point;
            // }
        }
        else {
            navMeshDestination = null;
            inactiveWaypointIndicator.SetActive(false);
        }


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
        if (interactable)
        {
            Vector3 interactPoint = interactable.Front
                ? interactable.Front.position
                // if the interactable did not specify
                // front, just find the closest point on its surface
                : Physics.ClosestPoint(
                    transform.position,
                    interactableCollider,
                    interactableCollider.transform.position,
                    interactableCollider.transform.rotation
                );

            if (Vector3.Distance(interactPoint.ProjectXZ(), transform.position.ProjectXZ()) > interactDistance) {
                Player.Instance.Navigation.NavigateTo(interactPoint);
            }
            else
            {
                interactable.Interact();
            }
        }

        if(navMeshDestination.HasValue)
        {
            Player.Instance.Navigation.NavigateTo(navMeshDestination.Value);
        }
    }
}
