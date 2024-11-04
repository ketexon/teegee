using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UI;

public class OutlineUI : MonoBehaviour
{
    [SerializeField] RawImage rawImage;
    [SerializeField] new Camera camera;

    void Reset()
    {
        rawImage = GetComponent<RawImage>();
    }

    RectTransform rectTransform;

    RenderTexture rt = null;

    Rect lastRect;

    void Awake()
    {
        rectTransform = transform as RectTransform;
    }

    void Start()
    {
        RecreateRenderTexture();
    }

    void Update()
    {
        if (lastRect != rectTransform.rect)
        {
            RecreateRenderTexture();
        }
    }

    void RecreateRenderTexture()
    {
        lastRect = rectTransform.rect;

        if(rt){
            rt.Release();
        }

        rt = new((int) lastRect.width, (int)lastRect.height, 16);
        rt.Create();

        rawImage.texture = rt;
        camera.targetTexture = rt;
    }
}
