def COLOR_MAP = ['SUCCESS': 'good', 'FAILURE': 'danger', 'UNSTABLE': 'danger', 'ABORTED': 'danger']

node {
    def rustatic
    def app

    environment {
        PATH = "$HOME/.cargo/bin/:$PATH"
    }

    stage('Checkout') {
        checkout scm

        withCredentials([usernamePassword( credentialsId: 'registry-joseluisq-net', usernameVariable: 'USERNAME', passwordVariable: 'PASSWORD')]) {
            try {
                docker.withRegistry('https://registry.joseluisq.net', 'registry-joseluisq-net') {
                    rustatic = docker.image('registry.joseluisq.net/rustatic:latest')
                }
            } catch (err) {
                error('Checkout failed!')
            }
        }
    }

    stage('Init') {
        rustatic.inside {
            sh 'rustc --version'
            sh 'cargo --version'
            sh 'rustup --version'

            sh 'echo'
            sh 'pwd'
            sh 'ls -lah'
        }
    }

    stage('Test') {
        sh 'echo "There are no tests at the moment!"'
    }

    stage('Build') {
        echo "Running ${env.BUILD_ID} on ${env.JENKINS_URL}"
        
        rustatic.inside {
            echo "PATH is: $PATH"

            sh 'rustatic $(pwd -P)'
        }
    }

    // stage('Deploy') {
    //     when {
    //         expression {
    //         currentBuild.result == null || currentBuild.result == 'SUCCESS'
    //         }
    //     }
    //     steps {
    //         sh 'echo "Push current docker image..........."'
    //     }
    // }

    // post {
    //     success {
    //         withCredentials([string(credentialsId: 'slack-token', variable: 'slackCredentials')]) {
    //             slackSend teamDomain: 'quintanaio',
    //                 token: slackCredentials, 
    //                 channel: '#jenkins',
    //                 color: 'good',
    //                 message: "*${currentBuild.currentResult}:* Job ${env.JOB_NAME} build ${env.BUILD_NUMBER}\n More info at: ${env.BUILD_URL}"
    //         }
    //     }
    //     failure {
    //         withCredentials([string(credentialsId: 'slack-token', variable: 'slackCredentials')]) {
    //             slackSend teamDomain: 'quintanaio',
    //                 token: slackCredentials, 
    //                 channel: '#jenkins',
    //                 color: COLOR_MAP[currentBuild.currentResult],
    //                 message: "*${currentBuild.currentResult}:* Job ${env.JOB_NAME} build ${env.BUILD_NUMBER}\n More info at: ${env.BUILD_URL}"
    //         }
    //     }
    // }
}
