# anypod

A quick way to get a random pod's name for a given Deployment, StatefulSet, or DaemoneSet 

## Motivation 

I often find myself needing to exec into a running pod to check out something in situ or 
to run a command in one of the pods of some Deployment.  It does not matter at all WHICH pod 
I choose, I just need one of the current pods.  

I got tired of having to run `kubectl -n whatever get pods | grep my-deployment-prefix` and 
then cut-n-paste that into what I'm trying to do.  


## Example 

If we have Deployment named `ingress-nginx-controller` in the `nginx-ingress` namespace and we run:

```shell
$ anypod -n nginx-ingress ingress 
```

It will output the name of a single pod running from that Deployment like so: 

```shell
$ anypod -n nginx-ingress ingress 
ingress-nginx-controller-69fbfb4bfd-dvn8d
```

## Usage

**NOTE:** All queries are prefix queries.  So if you run `anypod f` it will use the first workload it finds that 
starts with the letter 'f'


```shell
# Return the name of the nginx Deployment in the current namespace 
$ anypod ingress
ingress-nginx-controller-69fbfb4bfd-dvn8d

# Return the name of the nginx Deployment in the `other` namespace
$ anypod --namespace other nginx
nginx-79f798ccd6-j7d6s

# or 

$ anypod -n other nginx
nginx-79f798ccd6-j7d6s

# Specifically look for statefulset name
$ anypod statefulset/postgresql

# Specifically look for daemonset name
$ anypod daemonset/dd-agent
```

In the event `anypod` does not find any matching workloads it will tell you 
the namespace it is currently searching in and will list all of the various workloads it 
DID find so you can re-run your query. 

