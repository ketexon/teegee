using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class LoadingScreen : MonoBehaviour
{
    [SerializeField]
    TMPro.TMP_Text tmp;

    int nPeriods = 0;
    Coroutine coro;

    IEnumerator AnimationCoro(){
        while(true){
            var periods = new string('.', nPeriods);
            tmp.text = $"Loading{periods}";
            yield return new WaitForSeconds(1);
        }
    }

    public void OnEnable(){
        nPeriods = 0;
        coro = StartCoroutine(AnimationCoro());
    }

    public void OnDisable(){
        StopCoroutine(coro);
        coro = null;
    }
}
