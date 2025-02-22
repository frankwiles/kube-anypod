# anypod

A quick way to get a random pod's name for a given Deployment, StatefulSet, or DaemoneSet 

## Motivation 

I often find myself needing to exec into a running pod to check out something in situ or 
to run a command in one of the pods of some Deployment.  It does not matter at all WHICH pod 
I choose, I just need one of the current pods.  

I got tired of having to run `kubectl -n whatever get pods | grep my-deployment-prefix` and 
then cut-n-paste that into what I'm trying to do.  


## Usage

```shell
# Return the name of the nginx Deployment in the current namespace 
$ anypod nginx

# Return the name of the nginx Deployment in the `other` namespace
$ anypod --namespace other nginx

# or 

$ anypod -n other nginx

# Specifically look for statefulset name
$ anypod statefulset/postgresql

# Specifically look for daemonset name
$ anypod daemonset/dd-agent
```
