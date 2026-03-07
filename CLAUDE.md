# yu-sdk


# 目录结构
- go  - go语言的sdk包
- rust  - rust语言的sdk包
- js   - js语言的sdk包
- python  - python语言的sdk包
- .gitignore 把各个语言该忽略的文件夹和二进制文件都忽略掉

# 描述
这个项目是为了给 github.com/yu-org/yu 这个项目开发多语言的SDK所用，需要各类语言
都能调用yu的writing/reading接口，可以模仿 yu-org/yu/example/client
里调用的方式，将调用封装成各类语言的SDK。  
要实现的功能有： 向yu链上发起 writing和reading的调用请求，以及订阅event。  
按照目录结构的方式，把每个语言对应的SDK放在每个目录下，并且都将使用方式写在
各自目录的readme里，并且还要构建好测试代码，测试代码就用yu/example/client里
的示例来启动一个带有poa和asset的tripod的链，然后运行测试代码就可以了。   

# CI/CD  
使用github workflow进行CICD，如果是testchain/里面的代码有变更，则所有语言的sdk
都需要运行一遍测试；其他情况，如果每个语言的sdk有代码变更，只运行自己语言的这份SDK的测试，
CICD忽略所有 *.md文件 的变更。 测试的时候，testchain里面，只需要链运行半分钟即可通过yu的kernel.Stop()来停止链


