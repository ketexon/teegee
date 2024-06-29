using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class Pinpad : Terminal
{
    [SerializeField] byte[] code;
    [SerializeField] Door door;
    [SerializeField] AudioSource successAudioSource;
    [SerializeField] AudioSource failureAudioSource;

    protected override void OnTerminate(IMessage message)
    {
        if (message is not UnlockDoorMessage unlockDoorMessage)
        {
            Debug.LogWarning("Pinpad message is not the right type.");
        }
        else
        {
            if(code.Length != unlockDoorMessage.Code.Length)
            {
                Debug.LogError("Codes are of different lengths.");
                return;
            }

            bool equal = true;
            for(int i = 0; i < code.Length; ++i)
            {
                equal = unlockDoorMessage.Code[i] == code[i];
                if (!equal) break;
            }

            if(equal)
            {
                door.Open();
                successAudioSource.Play();
            }
            else
            {
                failureAudioSource.Play();
            }
        }
    }
}
