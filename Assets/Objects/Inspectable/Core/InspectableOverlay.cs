using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.InputSystem;
using UnityEngine.UI;

public class InspectableOverlay : SingletonMonoBehaviour<InspectableOverlay>
{
    [SerializeField] Canvas canvas;
    [SerializeField] Animator animator;
    [SerializeField, Min(0)] float inspectingObjectLerpMult = 10;

    [Header("Rendering")]
    [SerializeField] RawImage rawImage;
    [SerializeField] new Camera camera;
    [SerializeField] float defaultOrthographicSize = 5;
    [SerializeField, Min(0)] float minOrhographicSize = 0.1f;
    [SerializeField] float maxOrthographicSize = 10;

    [Header("Input")]
    [SerializeField] InputActionReference rotateAction;
    [SerializeField] InputActionReference zoomAction;
    [SerializeField] InputActionReference pointAction;
    [SerializeField] InputActionReference panAction;
    [SerializeField] InputActionReference exitAction;

    void Reset()
    {
        canvas = GetComponentInParent<Canvas>();
        rawImage = GetComponentInChildren<RawImage>();
        animator = GetComponentInChildren<Animator>();
    }

    RenderTexture rt = null;

    Rect lastRect;

    bool visible = false;

    GameObject inspectingObject = null;

    Vector3 inspectingObjectTargetPosition;
    float targetOrthographicSize;

    Vector2 mousePosition;
    Vector2 mousePx;

    public void Inspect(Inspectable inspectable)
    {
        visible = true;
        animator.SetBool("visible", true);

        camera.enabled = true;
        camera.orthographicSize = defaultOrthographicSize;
        targetOrthographicSize = defaultOrthographicSize;

        inspectingObject = Instantiate(
            inspectable.InspectPrefab,
            camera.transform
        );
        inspectingObjectTargetPosition = inspectingObject.transform.position;

        Player.Instance.Input.SwitchCurrentActionMap("Inspect");
    }

    public void StopInspect()
    {
        if (!visible) return;
        animator.SetBool("visible", false);
    }

    void Start()
    {
        lastRect = rawImage.rectTransform.rect;
        rt = new((int)lastRect.width, (int)lastRect.height, 0);
        rt.Create();

        camera.targetTexture = rt;
        rawImage.texture = rt;
    }

    void OnEnable()
    {
        pointAction.action.performed += OnPoint;

        rotateAction.action.performed += OnRotate;
        panAction.action.started += OnPan;
        panAction.action.performed += OnPan;
        zoomAction.action.performed += OnZoom;
        exitAction.action.performed += OnExit;
    }

    void OnDisable()
    {
        pointAction.action.performed -= OnPoint;

        rotateAction.action.performed -= OnRotate;
        panAction.action.started -= OnPan;
        panAction.action.performed -= OnPan;
        zoomAction.action.performed -= OnZoom;
        exitAction.action.performed -= OnExit;
    }

    void Update()
    {
        if (!visible) return;
        Debug.Log(rawImage.rectTransform.rect);
        if(rawImage.rectTransform.rect != lastRect)
        {
            lastRect = rawImage.rectTransform.rect;
            ResizeRenderTexture();
        }

        inspectingObject.transform.position = Vector3.Lerp(
            inspectingObject.transform.position,
            inspectingObjectTargetPosition,
            Time.deltaTime * inspectingObjectLerpMult
        );

        camera.orthographicSize = Mathf.Lerp(
            camera.orthographicSize,
            targetOrthographicSize,
            Time.deltaTime * inspectingObjectLerpMult
        );
    }

    override protected void OnDestroy()
    {
        base.OnDestroy();
        if(rt) Destroy(rt);
    }

    #region Rendering
    void ResizeRenderTexture()
    {
        rt.Release();
        rt.width = (int) lastRect.width;
        rt.height = (int) lastRect.height;
        rt.Create();
    }
    #endregion

    #region Input
    void OnPoint(InputAction.CallbackContext ctx)
    {
        mousePx = ctx.ReadValue<Vector2>();
        mousePosition = camera.ScreenToWorldPoint(mousePx);
    }

    void OnRotate(InputAction.CallbackContext ctx)
    {
        var value = ctx.ReadValue<Vector2>();
        inspectingObject.transform.rotation =
            Quaternion.Euler(value.y, -value.x, 0)
            * inspectingObject.transform.rotation;
    }

    Vector2 panStartMousePosition;
    Vector3 panStartTransformPosition;

    void OnPan(InputAction.CallbackContext ctx)
    {
        var mousePx = ctx.ReadValue<Vector2>();
        if (ctx.started)
        {
            panStartMousePosition = camera.ScreenToWorldPoint(mousePx);
            panStartTransformPosition = inspectingObjectTargetPosition;
        }
        else if (ctx.performed)
        {
            var curMousePosition = (Vector2)camera.ScreenToWorldPoint(mousePx);
            inspectingObjectTargetPosition =
                panStartTransformPosition + (Vector3)(curMousePosition - panStartMousePosition);
        }
    }

    void OnZoom(InputAction.CallbackContext ctx)
    {
        var mouseObjectDisplacement = (Vector2)inspectingObjectTargetPosition - mousePosition;

        targetOrthographicSize *= Mathf.Pow(2, ctx.ReadValue<float>());
        targetOrthographicSize = Mathf.Clamp(targetOrthographicSize, minOrhographicSize, maxOrthographicSize);

        var oldOrthographicSize = camera.orthographicSize;
        camera.orthographicSize = targetOrthographicSize;

        Vector2 newMousePosition = camera.ScreenToWorldPoint(mousePx);
        var newMouseObjectDisplacement = (Vector2)inspectingObjectTargetPosition - newMousePosition;

        var displacementDelta = newMouseObjectDisplacement - mouseObjectDisplacement;

        inspectingObjectTargetPosition -= (Vector3) displacementDelta;

        camera.orthographicSize = oldOrthographicSize;
        mousePosition = newMousePosition;
    }

    void OnExit(InputAction.CallbackContext ctx)
    {
        StopInspect();
    }
    #endregion

    #region Animator callbacks
    [System.Diagnostics.CodeAnalysis.SuppressMessage(
        "CodeQuality",
        "IDE0051:Remove unused private members",
        Justification = "Unity Editor Message"
    )]
    void OnUIHidden()
    {
        visible = false;
        camera.enabled = false;

        Destroy(inspectingObject);
        Player.Instance.Input.SwitchCurrentActionMap("Player");
    }
    #endregion
}
