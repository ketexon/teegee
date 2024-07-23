using System.Collections;
using System.Collections.Generic;
using Unity.VisualScripting;
using UnityEngine;

public class Pinpad : Terminal
{
    [SerializeField] byte[] code;
    [SerializeField] Door door;
    [SerializeField] AudioSource successAudioSource;
    [SerializeField] AudioSource failureAudioSource;
    [SerializeField] AudioSource clickAudioSource;

    bool unlocked = false;

    protected override void OnTerminate()
    {
        base.OnTerminate();

        //if (unlocked)
        //{
        //    door.Open();
        //    successAudioSource.Play();
        //}
        //else
        //{
        //    failureAudioSource.Play();
        //}
    }

    protected override void OnMessage(IPC.IMessage message)
    {
        base.OnMessage(message);

        if (message is IPC.UnlockDoorMessage unlockDoorMessage)
        {
            if(code.Length != unlockDoorMessage.Code.Length)
            {
                Debug.LogError("Codes are of different lengths.");
                return;
            }

            foreach(var b in unlockDoorMessage.Code)
            {
                Debug.Log(b);
            }

            unlocked = true;
            for(int i = 0; i < code.Length; ++i)
            {
                unlocked = unlockDoorMessage.Code[i] == code[i];
                if (!unlocked) break;
            }

            if (unlocked)
            {
                door.Open();
                successAudioSource.Play();
            }
            else
            {
                failureAudioSource.Play();
            }
        }
        else if(message is IPC.PlaySfxMessage playSfxMessage)
        {
            clickAudioSource.Play();
        }
    }
}
