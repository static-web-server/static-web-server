def COLOR_MAP = ['SUCCESS': 'good', 'FAILURE': 'danger', 'UNSTABLE': 'danger', 'ABORTED': 'danger']

pipeline {
    agent {
        docker {
            image 'rust:1.37-slim'
        }
    }

    environment {
        CI = 'true'
        BUILD_USER = ''
    }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Init') {
            steps {
                sh 'rustc --version'
                sh 'cargo --version'
                sh 'rustup target add x86_64-unknown-linux-musl'
                sh 'rustup show'
            }
        }

        stage('Test') {
            steps {
                sh 'echo "There are no tests at the moment!"'
            }
        }

        stage('Build') {
            steps {
                echo "Running ${env.BUILD_ID} on ${env.JENKINS_URL}"
                sh './build.sh'
            }
        }

        stage('Deploy') {
            when {
              expression {
                currentBuild.result == null || currentBuild.result == 'SUCCESS'
              }
            }
            steps {
                // sh 'cargo release --skip-push --skip-publish beta'
                sh 'echo "Push current docker image..........."'
            }
        }
    }

    post {
        success {
            withCredentials([string(credentialsId: 'slack-token', variable: 'slackCredentials')]) {
                slackSend teamDomain: 'quintanaio',
                    token: slackCredentials, 
                    channel: '#jenkins',
                    color: 'good',
                    message: "*${currentBuild.currentResult}:* Job ${env.JOB_NAME} build ${env.BUILD_NUMBER} by ${env.BUILD_USER}\n More info at: ${env.BUILD_URL}"
            }
        }
        failure {
            withCredentials([string(credentialsId: 'slack-token', variable: 'slackCredentials')]) {
                slackSend teamDomain: 'quintanaio',
                    token: slackCredentials, 
                    channel: '#jenkins',
                    color: COLOR_MAP[currentBuild.currentResult],
                    message: "*${currentBuild.currentResult}:* Job ${env.JOB_NAME} build ${env.BUILD_NUMBER} by ${env.BUILD_USER}\n More info at: ${env.BUILD_URL}"
            }
        }
    }
}
